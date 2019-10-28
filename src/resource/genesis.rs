use serde::Serialize;
use rocket::request::{Form, FromForm};
use rocket::State;
use rocket_contrib::templates::Template;

use nimiq_hash::Blake2bHash;
use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_keys::Address;

use crate::resource::ResourceRenderer;
use crate::albatross::Albatross;
use crate::utils::serialize_with_format;
use nimiq_primitives::networks::NetworkId;


#[derive(Serialize)]
pub struct GenesisInfo {
    #[serde(serialize_with = "serialize_with_format")]
    genesis_hash: Blake2bHash,
    #[serde(serialize_with = "serialize_with_format")]
    network_id: NetworkId,

    #[serde(serialize_with = "serialize_with_format")]
    staking_contract: Address,
}

impl From<&NetworkInfo> for GenesisInfo {
    fn from(network_info: &NetworkInfo) -> Self {
        Self {
            genesis_hash: network_info.genesis_hash().clone(),
            network_id: network_info.network_id(),
            staking_contract: network_info.validator_registry_address()
                .expect("Expected validator registry address to be set")
                .clone(),
        }
    }
}

impl From<NetworkInfo> for GenesisInfo {
    fn from(network_info: NetworkInfo) -> Self {
        Self::from(&network_info)
    }
}

#[get("/genesis-info")]
pub fn get_genesis(albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Template {
    renderer.render("genesis", &albatross.genesis_info, &albatross)
}
