use serde::Serialize;

use nimiq_transaction::{Transaction, TransactionFlags};
use nimiq_keys::Address;
use nimiq_primitives::account::AccountType;
use nimiq_primitives::coin::Coin;
use nimiq_hash::{Blake2bHash, Hash};

use crate::utils::*;


#[derive(Clone, Debug, Serialize)]
pub struct TransactionInfo {
    short_hash: String,
    #[serde(serialize_with = "serialize_with_beserial")]
    txid: Blake2bHash,
    is_contract_creation: bool,

    #[serde(serialize_with = "serialize_special_account_type")]
    pub sender_type: AccountType,
    #[serde(serialize_with = "serialize_address")]
    pub sender: Address,

    #[serde(serialize_with = "serialize_special_account_type")]
    pub recipient_type: AccountType,
    #[serde(serialize_with = "serialize_address")]
    pub recipient: Address,

    #[serde(serialize_with = "serialize_coin_formatted")]
    pub value: Coin,
    #[serde(serialize_with = "serialize_coin_formatted")]
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
            sender: transaction.sender,
            recipient_type: transaction.recipient_type,
            recipient: transaction.recipient,
            value: transaction.value,
            fee: transaction.value,
            validity_start_height: transaction.validity_start_height,
            data: transaction.data,
            proof: transaction.proof,
        }
    }
}