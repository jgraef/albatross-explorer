use serde::Serialize;

use nimiq_hash::Blake2bHash;
use nimiq_network_primitives::networks::NetworkInfo;

use crate::utils::serialize_blake2b;


#[derive(Serialize)]
pub struct GenesisInfo {
    #[serde(serialize_with = "serialize_blake2b")]
    genesis_hash: Blake2bHash,
}

impl From<&NetworkInfo> for GenesisInfo {
    fn from(network_info: &NetworkInfo) -> Self {
        Self {
            genesis_hash: network_info.genesis_hash().clone()
        }
    }
}

impl From<NetworkInfo> for GenesisInfo {
    fn from(network_info: NetworkInfo) -> Self {
        Self::from(&network_info)
    }
}