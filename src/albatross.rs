use std::sync::Arc;
use std::str::{FromStr, Utf8Error};
use std::fmt::{Display, Error as DisplayError, Formatter};
use std::cmp::Ordering;

use rocket::request::FromParam;
use rocket::http::RawStr;

use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_primitives::networks::NetworkId;
use nimiq_hash::Blake2bHash;
use nimiq_keys::{Address, PublicKey};
use nimiq_blockchain_albatross::chain_info::ChainInfo;
use nimiq_blockchain_albatross::blockchain::BlockchainEvent;
use nimiq_utils::observer::ListenerHandle;
use nimiq_transaction::Transaction;
use nimiq::client::{Client, Consensus};
use nimiq_validator::validator::Validator;

use crate::resource::genesis::GenesisInfo;
use crate::resource::block::BlockInfo;
use crate::resource::transaction::{TransactionInfo, Confirmation};
use crate::resource::account::{AccountInfo, AccountTransactionInfo};
use crate::resource::metadata::MetadataStore;



#[derive(Clone, Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Unrecognized identifier format: {}", _0)]
    Unrecognized(String),
    #[fail(display = "Invalid encoding: {}", _0)]
    InvalidEncoding(Utf8Error)
}

impl From<Utf8Error> for ParseError {
    fn from(e: Utf8Error) -> Self {
        Self::InvalidEncoding(e)
    }
}


#[derive(Clone, Debug)]
pub enum BlockIdentifier {
    Hash(Blake2bHash),
    Number(u32),
    Genesis
}

impl FromStr for BlockIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("genesis") {
            Ok(BlockIdentifier::Genesis)
        }
        else if let Ok(hash) = s.parse::<Blake2bHash>() {
            Ok(BlockIdentifier::Hash(hash))
        }
        else if let Ok(num) = s.parse::<u32>() {
            Ok(BlockIdentifier::Number(num))
        }
        else {
            Err(ParseError::Unrecognized(s.to_string()))
        }
    }
}

impl<'a> FromParam<'a> for BlockIdentifier {
    type Error = ParseError;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let param = param.url_decode()?;
        debug!("BlockIdentifier::from_param: {:?}", param);
        param.parse()
    }
}

impl Display for BlockIdentifier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), DisplayError> {
        match self {
            BlockIdentifier::Genesis => "genesis".fmt(f),
            BlockIdentifier::Number(block_number) => block_number.fmt(f),
            BlockIdentifier::Hash(block_hash) => block_hash.fmt(f),
        }
    }
}


#[derive(Clone, Debug)]
pub struct TransactionIdentifier(pub Blake2bHash);

impl FromStr for TransactionIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Blake2bHash>()
            .map_err(|_| ParseError::Unrecognized(s.to_string()))
            .map(|hash| TransactionIdentifier(hash))
    }
}

impl<'a> FromParam<'a> for TransactionIdentifier {
    type Error = ParseError;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let param = param.url_decode()?;
        debug!("TransactionIdentifier::from_param: {:?}", param);
        param.parse()
    }
}

impl Display for TransactionIdentifier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), DisplayError> {
        self.0.fmt(f)
    }
}

impl Into<Blake2bHash> for TransactionIdentifier {
    fn into(self) -> Blake2bHash {
        self.0
    }
}


#[derive(Clone, Debug)]
pub enum AccountIdentifier {
    Address(Address),
    PublicKey(PublicKey),
    Hash(Blake2bHash),
}

impl FromStr for AccountIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(address) = Address::from_user_friendly_address(s) {
            Ok(AccountIdentifier::Address(address))
        }
        else if let Ok(pubkey) = s.parse() {
            Ok(AccountIdentifier::PublicKey(pubkey))
        }
        else if let Ok(hash) = s.parse() {
            Ok(AccountIdentifier::Hash(hash))
        }
        else {
            Err(ParseError::Unrecognized(s.to_string()))
        }
    }
}

impl<'a> FromParam<'a> for AccountIdentifier {
    type Error = ParseError;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let param = param.url_decode()?;
        debug!("AccountIdentifier::from_param: {:?}", param);
        param.parse()
    }
}

impl From<AccountIdentifier> for Address {
    fn from(ident: AccountIdentifier) -> Self {
        match ident {
            AccountIdentifier::Address(address) => address,
            AccountIdentifier::PublicKey(pubkey) => Address::from(&pubkey),
            AccountIdentifier::Hash(hash) => Address::from(hash),
        }
    }
}

impl Display for AccountIdentifier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), DisplayError> {
        match self {
            AccountIdentifier::Address(address) => address.fmt(f),
            AccountIdentifier::PublicKey(pubkey) => pubkey.fmt(f),
            AccountIdentifier::Hash(hash) => hash.fmt(f),
        }
    }
}

pub enum AnyIdentifier {
    Block(BlockIdentifier),
    Transaction(TransactionIdentifier),
    Account(AccountIdentifier),
}

impl FromStr for AnyIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ident) = s.parse::<BlockIdentifier>() {
            Ok(AnyIdentifier::Block(ident))
        }
        else if let Ok(ident) = s.parse::<TransactionIdentifier>() {
            Ok(AnyIdentifier::Transaction(ident))
        }
        else if let Ok(ident) = s.parse::<AccountIdentifier>() {
            Ok(AnyIdentifier::Account(ident))
        }
        else {
            Err(ParseError::Unrecognized(s.to_string()))
        }
    }
}


struct Listeners {
    blockchain: ListenerHandle,
}

// Helper class bundling access to Albatross client and meta data store
pub struct Albatross {
    client: Client,
    // TODO: Remove? We have that ref in client now
    consensus: Arc<Consensus>,
    validator: Arc<Validator>,

    meta_store: Arc<MetadataStore>,

    pub genesis_info: GenesisInfo,
    pub genesis_hash: Blake2bHash,

    listeners: Listeners,
}

impl Albatross {
    pub fn new(client: Client, mut meta_store: MetadataStore) -> Self {
        let consensus = client.consensus();
        let validator = client.validator()
            .expect("Client must run as validator");
        let network_info = NetworkInfo::from_network_id(consensus.blockchain.network_id);
        let genesis_info = GenesisInfo::from(network_info);

        // hard-code some account aliases from the genesis config
        // TODO: We could also fetch all initial accounts from the genesis block and label them
        meta_store.set_account_alias(genesis_info.staking_contract.clone(), "Staking Contract");

        let meta_store = Arc::new(meta_store);
        let listeners = Self::init_listeners(&consensus, &meta_store);

        Self {
            client,
            consensus,
            validator,
            meta_store,
            genesis_info,
            genesis_hash: network_info.genesis_hash().clone(),
            listeners,
        }
    }

    fn init_listeners(consensus: &Arc<Consensus>, meta_store: &Arc<MetadataStore>) -> Listeners {
        let weak_consensus = Arc::downgrade(consensus);
        let weak_meta_store = Arc::downgrade(meta_store);
        let blockchain = consensus.blockchain.notifier.write().register(move |event: &BlockchainEvent| {
            let consensus = upgrade_weak!(weak_consensus);
            let meta_store = upgrade_weak!(weak_meta_store);
            match event {
                BlockchainEvent::Extended(hash) => {
                    let block = consensus.blockchain.get_block(&hash, true, true)
                        .unwrap_or_else(|| panic!("Extended with unknown block: {}", hash));
                    meta_store.push_block(&hash, &block);
                },
                BlockchainEvent::Finalized(hash) => {
                    let block = consensus.blockchain.get_block(&hash, true, true)
                        .unwrap_or_else(|| panic!("Extended with unknown block: {}", hash));
                    meta_store.push_block(&hash, &block);
                    meta_store.finalize_epoch(&hash, &block);
                },
                BlockchainEvent::Rebranched(_old_chain, _new_chain) => {
                    // TODO: Do we need to do anything here?
                }
            };
        });

        Listeners {
            blockchain,
        }
    }

    pub fn network_id(&self) -> NetworkId {
        self.consensus.blockchain.network_id
    }

    pub fn get_block_info(&self, ident: &BlockIdentifier) -> Option<BlockInfo> {
        self.get_chain_info(ident)
            .map(BlockInfo::from)
    }

    pub fn get_chain_info(&self, ident: &BlockIdentifier) -> Option<ChainInfo> {
        let chain_store = &self.consensus.blockchain.chain_store;
        match ident {
            BlockIdentifier::Hash(hash) => chain_store.get_chain_info(&hash, true, None),
            BlockIdentifier::Number(number) => chain_store.get_chain_info_at(*number, true, None),
            BlockIdentifier::Genesis => {
                chain_store.get_chain_info(&self.genesis_hash, true, None)
            },
        }
    }

    pub fn get_latest_blocks(&self, num: usize) -> Result<Vec<BlockInfo>, ()>{
        let chain_store = &self.consensus.blockchain.chain_store;
        let mut block_hash = chain_store
            .get_head(None)
            .ok_or(())?;
        let mut latest_blocks = Vec::new();

        for _ in 0..num {
            if let Some(chain_info) = chain_store.get_chain_info(&block_hash, true, None) {
                block_hash = chain_info.head.parent_hash().clone();
                latest_blocks.push(BlockInfo::from(chain_info));
            }
            else {
                break;
            }
        }

        Ok(latest_blocks)
    }

    pub fn get_account_info(&self, ident: &AccountIdentifier) -> AccountInfo {
        let address = ident.clone().into();
        let account = self.consensus.blockchain.state().accounts.get(&address, None);
        let alias = self.meta_store.get_account_alias(&address);

        let transactions = self.meta_store
            .get_account_transactions(&address)
            .into_iter()
            .map(|tx_meta| {
                let block_hash = tx_meta.block_hash.parse().expect("Failed to parse block hash");
                let block_tx_ids = tx_meta.tx_idx as usize;
                let (transaction, chain_info) = self.get_transaction_from_block(&block_hash, block_tx_ids)
                    .unwrap_or_else(|| panic!("Failed to fetch transaction from block chain: hash={}, tx_idx={}", block_hash, block_tx_ids));
                let confirmation = Confirmation::new(self.block_number(), chain_info.head.block_number());
                let tx_info = TransactionInfo::new(transaction, Some(block_hash), Some(block_tx_ids), confirmation, false);
                let is_sender = tx_info.sender_address == address;
                let is_recipient = tx_info.recipient_address == address;
                AccountTransactionInfo::new(tx_info, is_sender, is_recipient)
            })
            .rev() // reverse, since we want the newest to be first
            .collect();

        let genesis_balance = self.genesis_info.accounts.get(&address)
            .map(|account| account.balance());

        AccountInfo::new(address, account, alias, transactions, genesis_balance)
    }

    pub fn get_transaction_from_block(&self, block_hash: &Blake2bHash, tx_idx: usize) -> Option<(Transaction, ChainInfo)> {
        let chain_info = self.consensus.blockchain.chain_store
            .get_chain_info(block_hash, true, None)?;
        let transaction = chain_info.head.transactions()?.get(tx_idx)?.clone();
        Some((transaction, chain_info))
    }

    pub fn get_transaction_info(&self, ident: &TransactionIdentifier) -> Option<TransactionInfo> {
        let meta = self.meta_store.get_transaction(&ident.0)?;
        let block_hash = meta.block_hash.parse::<Blake2bHash>()
            .expect("Failed to parse Blake2b hash");
        let tx_idx = meta.tx_idx as usize;
        let (transaction, chain_info) = self.get_transaction_from_block(&block_hash, tx_idx)?;
        let confirmation = Confirmation::new(self.block_number(), chain_info.head.block_number());
        let info = TransactionInfo::new(transaction, Some(block_hash), Some(tx_idx), confirmation, false);
        Some(info)
    }

    pub fn get_head_hash(&self) -> Blake2bHash {
        self.consensus.blockchain.head_hash()
    }

    pub fn get_head_info(&self) -> BlockInfo {
        let ident = BlockIdentifier::Hash(self.get_head_hash());
        self.get_block_info(&ident).expect("Expected block chain to have a head")
    }

    pub fn block_number(&self) -> u32 {
        self.consensus.blockchain.block_number()
    }

    pub fn get_account_infos(&self) -> Vec<AccountInfo> {
        let addresses = self.meta_store.get_known_account_addresses();

        let mut account_infos: Vec<AccountInfo> = addresses.into_iter()
            .map(|address| self.get_account_info(&AccountIdentifier::Address(address)))
            .collect();

        // sort aliased first by name and then unaliased by address
        account_infos.sort_by(|a, b| {
            match (&a.primary_alias, &b.primary_alias, &a.address, &b.address) {
                (Some(a), Some(b), _, _) => a.cmp(b),
                (Some(_), None, _, _) => Ordering::Less,
                (None, Some(_), _, _) => Ordering::Greater,
                (None, None, a, b) => a.cmp(b),
            }
        });

        /*
        debug!("account info: num={}", account_infos.len());
        for account_info in &account_infos {
            debug!("account info: {:?}", account_info);
        }
        */

        account_infos
    }
}

impl Drop for Albatross {
    fn drop(&mut self) {
        self.consensus.blockchain.notifier.write().deregister(self.listeners.blockchain)
    }
}
