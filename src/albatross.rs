use std::sync::Arc;
use std::str::FromStr;
use std::fmt::{Display, Error as DisplayError, Formatter};

use rocket::request::FromParam;
use rocket::http::RawStr;

use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_primitives::networks::NetworkId;
use nimiq_consensus::Consensus as _Consensus;
use nimiq_consensus::AlbatrossConsensusProtocol;
use nimiq_hash::Blake2bHash;
use nimiq_keys::{Address, PublicKey};
use nimiq_block_albatross::Block;
use nimiq_blockchain_albatross::chain_info::ChainInfo;

use crate::resource::genesis::GenesisInfo;
use crate::resource::block::BlockInfo;
use crate::resource::transaction::TransactionInfo;
use crate::resource::account::AccountInfo;


// rename `Consensus` for us, since we only use Albatross and the Arc'd one
pub type Consensus = Arc<_Consensus<AlbatrossConsensusProtocol>>;


#[derive(Clone, Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Unrecognized identifier format: {}", _0)]
    Unrecognized(String)
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
        param.parse()
    }
}

impl Display for TransactionIdentifier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), DisplayError> {
        self.0.fmt(f)
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
        if let Ok(address) = s.parse() {
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






// Helper class bundling access to Albatross client and meta data store
pub struct Albatross {
    consensus: Consensus,
    pub genesis_info: GenesisInfo,
    pub genesis_hash: Blake2bHash,
}

impl Albatross {
    pub fn new(consensus: Consensus) -> Self {
        let network_info = NetworkInfo::from_network_id(consensus.blockchain.network_id);
        Self {
            consensus,
            genesis_info: GenesisInfo::from(network_info),
            genesis_hash: network_info.genesis_hash().clone(),
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
            if let Some(chain_info) = chain_store.get_chain_info(&block_hash, false, None) {
                block_hash = chain_info.head.parent_hash().clone();
                latest_blocks.push(BlockInfo::from(chain_info));
            }
            else {
                break;
            }
        }

        Ok(latest_blocks)
    }

    pub fn get_transaction_info(&self, ident: &TransactionIdentifier) -> Option<TransactionInfo> {
        // TODO: We need to store block_hash and transaction indices for tx hashes.
        // NOTE: The same transaction might appear in multiple blocks
        let _hash = &ident.0;
        None
    }

    pub fn get_account_info(&self, ident: &AccountIdentifier) -> Option<AccountInfo> {
        unimplemented!()
    }

    pub fn get_head_hash(&self) -> Blake2bHash {
        self.consensus.blockchain.head_hash()
    }

    pub fn get_head_info(&self) -> BlockInfo {
        let ident = BlockIdentifier::Hash(self.get_head_hash());
        self.get_block_info(&ident).expect("Expected block chain to have a head")
    }
}
