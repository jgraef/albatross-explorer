use std::sync::Arc;
use std::str::FromStr;

use rocket::request::FromParam;
use rocket::http::RawStr;

use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_primitives::networks::NetworkId;
use nimiq_consensus::Consensus as _Consensus;
use nimiq_consensus::AlbatrossConsensusProtocol;
use nimiq_hash::Blake2bHash;

use crate::resources::genesis::GenesisInfo;
use crate::resources::block::BlockInfo;


// rename `Consensus` for us, since we only use Albatross and the Arc'd one
pub type Consensus = Arc<_Consensus<AlbatrossConsensusProtocol>>;


#[derive(Clone, Debug, Fail)]
pub enum BlockIdentifierParseError {
    #[fail(display = "Block identifier unrecognized: {}", _0)]
    Unrecognized(String)
}

#[derive(Clone, Debug)]
pub enum BlockIdentifier {
    Hash(Blake2bHash),
    Number(u32),
    Genesis
}

impl FromStr for BlockIdentifier {
    type Err = BlockIdentifierParseError;

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
            Err(BlockIdentifierParseError::Unrecognized(s.to_string()))
        }
    }
}

impl<'a> FromParam<'a> for BlockIdentifier {
    type Error = BlockIdentifierParseError;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        param.parse()
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
        let chain_store = &self.consensus.blockchain.chain_store;
        let chain_info = match ident {
            BlockIdentifier::Hash(hash) => chain_store.get_chain_info(&hash, true, None),
            BlockIdentifier::Number(number) => chain_store.get_chain_info_at(*number, true, None),
            BlockIdentifier::Genesis => {
                chain_store.get_chain_info(&self.genesis_hash, true, None)
            },
        }?;
        Some(BlockInfo::from(chain_info))
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
}
