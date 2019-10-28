use std::sync::Arc;

use serde::Serialize;
use rocket_contrib::templates::Template;
use rocket::State;

use nimiq_transaction::{Transaction, TransactionFlags};
use nimiq_keys::Address;
use nimiq_primitives::account::AccountType;
use nimiq_primitives::coin::Coin;
use nimiq_hash::{Blake2bHash, Hash};
use nimiq_account::{Account, BasicAccount, HashedTimeLockedContract, VestingContract, StakingContract};
use nimiq_transaction::account::htlc_contract::{AnyHash, HashAlgorithm};
use nimiq_bls::bls12_381::CompressedPublicKey as BlsPublicKey;

use crate::resource::ResourceRenderer;
use crate::albatross::{Albatross, AccountIdentifier};
use crate::utils::{serialize_with_format, serialize_address, serialize_address_opt,
                   serialize_with_beserial};
use nimiq_network::websocket::stream::WebSocketState::Active;
use nimiq_network::connection::close_type::CloseType::InboundConnectionsBlocked;


#[derive(Clone, Debug, Serialize)]
pub struct BasicAccountInfo {
    #[serde(serialize_with = "serialize_with_format")]
    balance: Coin,
}

#[derive(Clone, Debug, Serialize)]
pub struct HTLCContractInfo {
    #[serde(serialize_with = "serialize_with_format")]
    balance: Coin,
    #[serde(serialize_with = "serialize_address")]
    sender: Address,
    #[serde(serialize_with = "serialize_address")]
    recipient: Address,
    #[serde(serialize_with = "serialize_with_format")]
    hash_algorithm: HashAlgorithm,
    #[serde(serialize_with = "serialize_with_format")]
    hash_root: AnyHash,
    hash_count: u8,
    timeout: u32,
    #[serde(serialize_with = "serialize_with_format")]
    total_amount: Coin,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActiveStakeInfo {
    #[serde(serialize_with = "serialize_with_format")]
    balance: Coin,
    #[serde(serialize_with = "serialize_address")]
    staker_address: Address,
    #[serde(serialize_with = "serialize_address_opt")]
    reward_address: Option<Address>,
    #[serde(serialize_with = "serialize_with_beserial")]
    validator_key: BlsPublicKey,
}

#[derive(Clone, Debug, Serialize)]
pub struct InactiveStakeInfo {
    #[serde(serialize_with = "serialize_with_format")]
    balance: Coin,
    retire_time: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct StakingContractInfo {
    #[serde(serialize_with = "serialize_with_format")]
    balance: Coin,
    active_stake: Vec<ActiveStakeInfo>,
    inactive_stake: Vec<InactiveStakeInfo>
}


#[derive(Clone, Debug, Serialize)]
pub struct VestingContractInfo {
    #[serde(serialize_with = "serialize_with_format")]
    pub balance: Coin,
    #[serde(serialize_with = "serialize_with_format")]
    pub owner: Address,
    pub start: u32,
    pub step_blocks: u32,
    #[serde(serialize_with = "serialize_with_format")]
    pub step_amount: Coin,
    #[serde(serialize_with = "serialize_with_format")]
    pub total_amount: Coin,
}


#[derive(Clone, Debug, Serialize)]
pub enum AccountData {
    Basic(BasicAccountInfo),
    HTLC(HTLCContractInfo),
    Staking(StakingContractInfo),
    Vesting(VestingContractInfo),
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountInfo {
    #[serde(serialize_with = "serialize_address")]
    address: Address,

    #[serde(serialize_with = "serialize_with_format")]
    account_type: AccountType,

    account_data: AccountData
}


impl AccountInfo {
    pub fn new(address: Address, account: Account) -> Self {
        let account_type = account.account_type();
        let account_data = match account {
            Account::Basic(basic) => AccountData::Basic(BasicAccountInfo {
                balance: basic.balance
            }),
            Account::HTLC(htlc) => AccountData::HTLC(HTLCContractInfo {
                balance: htlc.balance,
                sender: htlc.sender,
                recipient: htlc.recipient,
                hash_algorithm: htlc.hash_algorithm,
                hash_root: htlc.hash_root,
                hash_count: htlc.hash_count,
                timeout: htlc.timeout,
                total_amount: htlc.total_amount,
            }),
            Account::Staking(staking) => {
                // drop this mapping in order to be able to unwrap the Arc's
                drop(staking.active_stake_by_address);

                let active_stake = staking.active_stake_sorted.into_iter().map(|active| {
                    let active = Arc::try_unwrap(active).unwrap();
                    ActiveStakeInfo {
                        balance: active.balance,
                        staker_address: active.staker_address,
                        reward_address: active.reward_address,
                        validator_key: active.validator_key,
                    }
                }).collect();

                let inactive_stake = staking.inactive_stake_by_address.into_iter().map(|(_, inactive)| {
                    InactiveStakeInfo {
                        balance: inactive.balance,
                        retire_time: inactive.retire_time,
                    }
                }).collect();

                AccountData::Staking(StakingContractInfo {
                    balance: staking.balance,
                    active_stake,
                    inactive_stake,
                })
            },
            Account::Vesting(vesting) => {
                AccountData::Vesting(VestingContractInfo {
                    balance: vesting.balance,
                    owner: vesting.owner,
                    start: vesting.start,
                    step_blocks: vesting.step_blocks,
                    step_amount: vesting.step_amount,
                    total_amount: vesting.total_amount,
                })
            }
        };


        Self {
            address,
            account_type,
            account_data,
        }
    }
}


#[get("/account/<ident>")]
pub fn get_account(ident: AccountIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    let account_info = albatross.get_account_info(&ident)?;
    Some(renderer.render("account", account_info, &albatross))
}
