use std::sync::Arc;

use serde::Serialize;
use rocket_contrib::templates::Template;
use rocket::State;

use nimiq_keys::Address;
use nimiq_primitives::account::AccountType;
use nimiq_primitives::coin::Coin;
use nimiq_account::Account;
use nimiq_transaction::account::htlc_contract::{AnyHash, HashAlgorithm};
use nimiq_bls::bls12_381::CompressedPublicKey as BlsPublicKey;

use crate::resource::ResourceRenderer;
use crate::albatross::{Albatross, AccountIdentifier};
use crate::utils::{serialize_with_format, serialize_address, serialize_address_opt,
                   serialize_with_beserial, short_validator_key, serialize_with_format_opt};
use crate::resource::transaction::TransactionInfo;


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

    short_validator_key: String,
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
#[serde(untagged)]
pub enum AccountData {
    Basic(BasicAccountInfo),
    HTLC(HTLCContractInfo),
    Staking(StakingContractInfo),
    Vesting(VestingContractInfo),
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountInfo {
    #[serde(serialize_with = "serialize_address")]
    pub address: Address,

    #[serde(serialize_with = "serialize_with_format")]
    pub account_type: AccountType,
    is_basic: bool,
    is_htlc: bool,
    is_vesting: bool,
    is_staking: bool,

    is_empty: bool,

    account_data: AccountData,
    transactions: Vec<AccountTransactionInfo>,

    pub primary_alias: Option<String>,
    aliases: Vec<String>,

    #[serde(serialize_with = "serialize_with_format_opt")]
    genesis_balance: Option<Coin>,
}


impl AccountInfo {
    // TODO: also pass in the active stakes if the address is a reward or staker address
    pub fn new(address: Address, account: Account, primary_alias: Option<String>, transactions: Vec<AccountTransactionInfo>, genesis_balance: Option<Coin>) -> Self {
        let account_type = account.account_type();
        let is_empty = account.is_initial();

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

                let active_stake: Vec<ActiveStakeInfo> = staking.active_stake_sorted.into_iter().map(|active| {
                    let active = Arc::try_unwrap(active).unwrap();
                    let short_validator_key = short_validator_key(&active.validator_key);
                    ActiveStakeInfo {
                        balance: active.balance,
                        staker_address: active.staker_address,
                        reward_address: active.reward_address,
                        validator_key: active.validator_key,
                        short_validator_key,
                    }
                }).collect();

                let inactive_stake: Vec<InactiveStakeInfo> = staking.inactive_stake_by_address.into_iter().map(|(_, inactive)| {
                    InactiveStakeInfo {
                        balance: inactive.balance,
                        retire_time: inactive.retire_time,
                    }
                }).collect();

                debug!("active stakes: {}", active_stake.len());
                for stake in &active_stake {
                    debug!("active stake: {:?}", stake);
                }

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
            is_basic: account_type == AccountType::Basic,
            is_htlc: account_type == AccountType::HTLC,
            is_vesting: account_type == AccountType::Vesting,
            is_staking: account_type == AccountType::Staking,
            is_empty,
            account_data,
            transactions,
            primary_alias,
            // TODO This will later be filled form aliases stored in the database
            aliases: vec![],
            genesis_balance,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountTransactionInfo {
    #[serde(flatten)]
    tx: TransactionInfo,
    is_sender: bool,
    is_recipient: bool,
}

impl AccountTransactionInfo {
    pub fn new(tx: TransactionInfo, is_sender: bool, is_recipient: bool) -> Self {
        Self {
            tx,
            is_sender,
            is_recipient,
        }
    }
}


#[get("/account/<ident>")]
pub fn get_account(ident: AccountIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    let account_info = albatross.get_account_info(&ident);
    Some(renderer.render("account", account_info, &albatross))
}

#[get["/accounts"]]
pub fn get_accounts(albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Template {
    let account_infos = albatross.get_account_infos();
    renderer.render("accounts", account_infos, &albatross)
}
