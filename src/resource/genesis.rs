use std::collections::BTreeMap;

use serde::Serialize;
use rocket::State;
use rocket_contrib::templates::Template;

use nimiq_hash::Blake2bHash;
use nimiq_network_primitives::networks::NetworkInfo;
use nimiq_keys::Address;
use nimiq_account::Account;

use crate::resource::ResourceRenderer;
use crate::albatross::Albatross;
use crate::utils::serialize_with_format;
use nimiq_primitives::networks::NetworkId;


#[derive(Clone, Debug, Serialize)]
pub struct GenesisInfo {
    #[serde(serialize_with = "serialize_with_format")]
    pub genesis_hash: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format")]
    pub network_id: NetworkId,

    #[serde(serialize_with = "serialize_with_format")]
    pub staking_contract: Address,

    #[serde(skip)]
    pub accounts: BTreeMap<Address, Account>,
}

impl From<&NetworkInfo> for GenesisInfo {
    fn from(network_info: &NetworkInfo) -> Self {
        let mut accounts = BTreeMap::new();
        for (address, account) in network_info.genesis_accounts() {
            accounts.insert(address, account);
        }

        Self {
            genesis_hash: network_info.genesis_hash().clone(),
            network_id: network_info.network_id(),
            staking_contract: network_info.validator_registry_address()
                .expect("Expected validator registry address to be set")
                .clone(),
            accounts,
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
