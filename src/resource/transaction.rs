use serde::{Serialize, Serializer};
use rocket_contrib::templates::Template;
use rocket::State;

use nimiq_transaction::{Transaction, TransactionFlags};
use nimiq_keys::Address;
use nimiq_primitives::account::AccountType;
use nimiq_primitives::coin::Coin;
use nimiq_hash::{Blake2bHash, Hash};
use nimiq_primitives::policy::epoch_at;

use crate::resource::{ResourceRenderer, Download};
use crate::albatross::{Albatross, TransactionIdentifier};
use crate::utils::{serialize_with_format, serialize_with_hex, short_hash, serialize_address,
                   serialize_with_format_opt, serialize_special_account_type};


#[derive(Clone, Debug, Serialize)]
pub struct TransactionData {

}

#[derive(Clone, Debug, Serialize)]
// TODO: interpret transaction data
pub struct TransactionInfo {
    /// Short transaction ID
    short_txid: String,

    /// Transaction ID
    #[serde(serialize_with = "serialize_with_format")]
    txid: Blake2bHash,

    /// If this transaction was a contract creation
    is_contract_creation: bool,

    /// Short hash of that block
    block_short_hash: Option<String>,

    /// Hash of block in which the transaction was included
    #[serde(serialize_with = "serialize_with_format_opt")]
    block_hash: Option<Blake2bHash>,

    /// Index into the transactions of that block
    block_tx_idx: Option<usize>,

    /// How many blocks has there been since the inclusion (the inclusion block counts aswell)
    confirmation: Confirmation,

    /// Whether the transaction is in the mempool
    is_in_mempool: bool,

    /// Sender account type
    #[serde(serialize_with = "serialize_special_account_type")]
    pub sender_type: AccountType,

    /// Sender address
    #[serde(serialize_with = "serialize_address")]
    pub sender_address: Address,

    /// Recipient type
    #[serde(serialize_with = "serialize_special_account_type")]
    pub recipient_type: AccountType,

    /// Recipient address
    #[serde(serialize_with = "serialize_address")]
    pub recipient_address: Address,

    /// Amount of NIM spent in this transaction
    #[serde(serialize_with = "serialize_with_format")]
    pub value: Coin,

    /// Fee spent on this transaction
    #[serde(serialize_with = "serialize_with_format")]
    pub fee: Coin,

    pub validity_start_height: u32,

    #[serde(serialize_with = "serialize_with_hex")]
    data_raw: Vec<u8>,
    data: Option<TransactionData>,

    #[serde(serialize_with = "serialize_with_hex")]
    proof: Vec<u8>,
}

impl From<Transaction> for TransactionInfo {
    fn from(transaction: Transaction) -> Self {
        TransactionInfo::new(transaction, None, None, Default::default(), false)
    }
}

impl TransactionInfo {
    pub fn new(transaction: Transaction, block_hash: Option<Blake2bHash>, block_tx_idx: Option<usize>, confirmation: Confirmation, is_in_mempool: bool) -> Self {
        let txid = transaction.hash();

        Self {
            short_txid: short_hash(&txid),
            txid,
            is_contract_creation: transaction.flags.contains(TransactionFlags::CONTRACT_CREATION),

            block_short_hash: block_hash.as_ref().map(|h| short_hash(h)),
            block_hash,
            block_tx_idx,
            confirmation,
            is_in_mempool,

            sender_type: transaction.sender_type,
            sender_address: transaction.sender,
            recipient_type: transaction.recipient_type,
            recipient_address: transaction.recipient,
            value: transaction.value,
            fee: transaction.fee,
            validity_start_height: transaction.validity_start_height,
            data_raw: transaction.data,
            data: None,
            proof: transaction.proof,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Confirmation {
    Blocks(u32),
    Final,
}

impl Confirmation {
    pub fn new(blockchain_height: u32, transaction_height: u32) -> Self {
        if epoch_at(blockchain_height) > epoch_at(transaction_height) {
            Confirmation::Final
        }
        else {
            let blocks = (blockchain_height + 1).saturating_sub(transaction_height);
            Confirmation::Blocks(blocks)
        }
    }
}

impl Serialize for Confirmation {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        match self {
            Confirmation::Blocks(n) => serializer.serialize_u32(*n),
            Confirmation::Final => serializer.serialize_str("final"),
        }
    }
}

impl Default for Confirmation {
    fn default() -> Self {
        Confirmation::Blocks(0)
    }
}


#[get("/transaction/<ident>")]
pub fn get_transaction(ident: TransactionIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    let block_info = albatross.get_transaction_info(&ident)?;
    Some(renderer.render("transaction", block_info, &albatross))
}

#[get("/transaction/<ident>/download")]
pub fn download_transaction(ident: TransactionIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Download, ()> {
    //let transaction = albatross.
    //renderer.download(chain_info.head).map_err(|e| warn!("Download failed: {}", e))
    let _ = (ident, albatross, renderer); // those variables are not unused ;)
    unimplemented!();
}
