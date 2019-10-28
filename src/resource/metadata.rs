use diesel::Connection;
use nimiq_block_albatross::Block;
use nimiq_transaction::Transaction;
use nimiq_hash::Blake2bHash;

use crate::resource::transaction::TransactionMeta;


#[derive(Clone, Debug, Queryable)]
pub struct AccountAlias {
    txid: Blake2bHash,
    alias: String,
}

#[derive(Clone, Debug, Queryable)]
pub struct TransactionMeta {

}


pub struct MetadataStore<C: Connection> {
    connection: C,
}

impl<C: Connection> MetadataStore<C> {
    pub fn new(connection: C) -> Self {
        Self {
            connection,
        }
    }

    pub fn push_transaction(transaction: &Transaction) {
        unimplemented!()
    }
}
