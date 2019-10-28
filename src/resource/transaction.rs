use serde::Serialize;
use rocket_contrib::templates::Template;
use rocket::State;

use nimiq_transaction::{Transaction, TransactionFlags};
use nimiq_keys::Address;
use nimiq_primitives::account::AccountType;
use nimiq_primitives::coin::Coin;
use nimiq_hash::{Blake2bHash, Hash};

use crate::resource::ResourceRenderer;
use crate::albatross::{Albatross, TransactionIdentifier};
use crate::utils::{serialize_with_format, serialize_with_format_opt, serialize_with_hex,
                   serialize_special_account_type, short_hash, serialize_address};



#[derive(Clone, Debug, Serialize)]
pub struct TransactionInfo {
    short_hash: String,
    #[serde(serialize_with = "serialize_with_format")]
    txid: Blake2bHash,
    is_contract_creation: bool,

    #[serde(serialize_with = "serialize_with_format")]
    pub sender_type: AccountType,
    #[serde(serialize_with = "serialize_address")]
    pub sender_address: Address,

    #[serde(serialize_with = "serialize_with_format")]
    pub recipient_type: AccountType,
    #[serde(serialize_with = "serialize_address")]
    pub recipient_address: Address,

    #[serde(serialize_with = "serialize_with_format")]
    pub value: Coin,
    #[serde(serialize_with = "serialize_with_format")]
    pub fee: Coin,

    pub validity_start_height: u32,

    #[serde(serialize_with = "serialize_with_hex")]
    data: Vec<u8>,
    #[serde(serialize_with = "serialize_with_hex")]
    proof: Vec<u8>,
}

impl From<Transaction> for TransactionInfo {
    fn from(transaction: Transaction) -> Self {
        let txid = transaction.hash();
        Self {
            short_hash: short_hash(&txid),
            txid,
            is_contract_creation: transaction.flags.contains(TransactionFlags::CONTRACT_CREATION),
            sender_type: transaction.sender_type,
            sender_address: transaction.sender,
            recipient_type: transaction.recipient_type,
            recipient_address: transaction.recipient,
            value: transaction.value,
            fee: transaction.value,
            validity_start_height: transaction.validity_start_height,
            data: transaction.data,
            proof: transaction.proof,
        }
    }
}


#[get("/transaction/<ident>")]
pub fn get_transaction(ident: TransactionIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    let block_info = albatross.get_transaction_info(&ident)?;
    Some(renderer.render("block", block_info, &albatross))
}
