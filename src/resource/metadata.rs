use std::fmt::Debug;
use std::collections::{HashMap, HashSet};

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::{Pool, PooledConnection};

use nimiq_block_albatross::{Block, ForkProof};
use nimiq_transaction::Transaction;
use nimiq_hash::{Blake2bHash, Hash};
use nimiq_keys::Address;

use crate::schema::{transactions, account_aliases};



#[derive(Clone, Debug, Insertable)]
#[table_name="account_aliases"]
struct NewAccountAlias {
    address: String,
    alias: String,
}

#[derive(Clone, Debug, Queryable)]
pub struct AccountAlias {
    id: i32,
    address: String,
    alias: String,
}

#[derive(Clone, Debug, Queryable)]
pub struct TransactionMeta {
    id: i32,
    pub txid: String,
    pub block_hash: String,
    pub block_number: i32, // useful for ordering
    pub tx_idx: i32,
    pub sender: String,
    pub recipient: String,
}

#[derive(Clone, Debug, Insertable)]
#[table_name="transactions"]
struct NewTransactionMeta {
    txid: String,
    block_hash: String,
    block_number: i32,
    tx_idx: i32,
    sender: String,
    recipient: String,
}


type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPoolConnection = PooledConnection<ConnectionManager<PgConnection>>;

// TODO: Nimiq has native support for a TransactionStore, that supports lookup by sender and
// recipient address
// We have access to the LMBD and could store stuff in there aswell. No need for diesel then, but
// need to implement serialization for metadata
pub struct MetadataStore {
    db_pool: PgPool,
    account_aliases: HashMap<Address, String>,
}

impl MetadataStore {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        // create database connection pool
        let manager = ConnectionManager::<PgConnection>::new(url.as_ref());
        let db_pool = Pool::builder().build(manager)
            .expect("Failed to create database connection pool");

        Self {
            db_pool,
            account_aliases: HashMap::new(),
        }
    }

    fn db_conn(&self) -> PgPoolConnection {
        self.db_pool.get()
            .unwrap_or_else(|e| panic!("Failed to get database connection from pool: {}", e))
    }

    pub fn finalize_epoch(&self, _block_hash: &Blake2bHash, block: &Block) {
        let _macro_block = block.unwrap_macro_ref();
        // TODO
    }

    pub fn push_block(&self, block_hash: &Blake2bHash, block: &Block) {
        match block {
            Block::Micro(micro_block) => {
                let extrinsics = micro_block.extrinsics.as_ref().expect("Micro block without extrinsics");

                for (i, transaction) in extrinsics.transactions.iter().enumerate() {
                    self.push_transaction(transaction, &block_hash, micro_block.header.block_number, i);
                }

                for (i, fork_proof) in extrinsics.fork_proofs.iter().enumerate() {
                    self.push_fork_proof(fork_proof, &block_hash, i);
                }

                // TODO also look into micro block justification to store information about view changes?
            },

            Block::Macro(_) => {
                // TODO: Has no transactions, but we could also store inherents in the DB
            },
        }
    }

    fn push_transaction(&self, transaction: &Transaction, block_hash: &Blake2bHash, block_number: u32, tx_idx: usize) -> TransactionMeta {
        let txid = transaction.hash::<Blake2bHash>();

        let tx_meta = NewTransactionMeta {
            txid: txid.to_hex(),
            block_hash: block_hash.to_hex(),
            block_number: block_number as i32,
            tx_idx: tx_idx as i32,
            sender: transaction.sender.to_user_friendly_address(),
            recipient: transaction.recipient.to_user_friendly_address(),
        };

        diesel::insert_into(transactions::table)
            .values(&tx_meta)
            .get_result(&self.db_conn())
            .expect("Failed to write transaction meta data to database")
    }

    fn push_fork_proof(&self, _fork_proof: &ForkProof, _block_hash: &Blake2bHash, _fork_proof_idx: usize) {
        // TODO Get the corresponding validator and store (validator_pubkey/fingerprint, block_hash, fork_proof_idx)
        // This way we can lookup for an validator if they've done forks in the past
    }

    /// Get secondary aliases from database - not used at the moment.
    pub fn get_account_aliases(&self, address: &Address) -> Vec<String> {
        use account_aliases::dsl;
        dsl::account_aliases.filter(dsl::address.eq(address.to_user_friendly_address()))
            .load::<AccountAlias>(&self.db_conn())
            .expect("Failed to fetch account aliases from database")
            .into_iter()
            .map(|alias| alias.alias)
            .collect()
    }

    /// Set in-memory primary alias
    pub fn set_account_alias<S: AsRef<str>>(&mut self, address: Address, alias: S) {
        self.account_aliases.insert(address, alias.as_ref().to_string());
    }

    /// Get in-memory primary alias
    pub fn get_account_alias(&self, address: &Address) -> Option<String> {
        self.account_aliases.get(address).cloned()
    }

    pub fn get_known_account_addresses(&self) -> Vec<Address> {
        use account_aliases::{dsl as dsl1};
        use transactions::{dsl as dsl2};

        let conn = self.db_conn();
        let mut addresses = HashSet::new();

        // get all addresses with aliases
        // TODO: We shouldn't use this, I think?
        for alias in dsl1::account_aliases
            .load::<AccountAlias>(&conn)
            .expect("Failed to fetch account aliases from database") {
            addresses.insert(alias.address);
        }

        // get all other accounts we saw transactions for
        for tx_meta in dsl2::transactions
            .load::<TransactionMeta>(&conn)
            .expect("Failed to fetch transaction meta data from database") {
            addresses.insert(tx_meta.sender);
            addresses.insert(tx_meta.recipient);
        }

        addresses.into_iter()
            .map(|address| {
                Address::from_user_friendly_address(&address)
                    .unwrap_or_else(|e| panic!("Failed to parse Address from database: {}", e))
            })
            .collect()
    }

    pub fn get_account_transactions(&self, address: &Address) -> Vec<TransactionMeta> {
        use transactions::dsl;
        let address = address.to_user_friendly_address();

        dsl::transactions
            .filter(dsl::recipient.eq(&address).or(dsl::sender.eq(&address)))
            .order((dsl::block_number.desc(), dsl::tx_idx.desc()))
            .load::<TransactionMeta>(&self.db_conn())
            .expect("Failed to fetch account transactions from database")
    }

    pub fn get_transaction(&self, txid: &Blake2bHash) -> Option<TransactionMeta> {
        use transactions::dsl;
        let txid = txid.to_hex();

        dsl::transactions
            .filter(dsl::txid.eq(&txid))
            .first::<TransactionMeta>(&self.db_conn())
            .ok()
    }
}
