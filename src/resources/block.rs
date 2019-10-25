use serde::Serialize;
use chrono::{DateTime, Utc};

use nimiq_hash::Blake2bHash;
use nimiq_block_albatross::Block;
use nimiq_blockchain_albatross::chain_info::ChainInfo;
use nimiq_bls::bls12_381::CompressedSignature;
use nimiq_primitives::policy::epoch_at;

use crate::utils::*;
use crate::resources::transaction::TransactionInfo;



#[derive(Clone, Debug, Serialize)]
pub struct BlockInfo {
    short_hash: String,
    is_macro: bool,
    epoch: u32,

    #[serde(serialize_with = "serialize_with_beserial")]
    block_hash: Blake2bHash,

    #[serde(serialize_with = "serialize_with_beserial_opt")]
    parent_hash: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_beserial_opt")]
    parent_macro_hash: Option<Blake2bHash>,

    block_number: u32,
    view_number: u32,

    timestamp: DateTime<Utc>,

    #[serde(serialize_with = "serialize_with_beserial")]
    seed: CompressedSignature,

    #[serde(serialize_with = "serialize_with_beserial")]
    state_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_beserial")]
    extrinsics_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_beserial_opt")]
    transaction_root: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_beserial_opt")]
    next_block_hash: Option<Blake2bHash>,

    transactions: Option<Vec<TransactionInfo>>,
}

impl From<ChainInfo> for BlockInfo {
    fn from(chain_info: ChainInfo) -> Self {
        let block_hash = chain_info.head.hash();

        match chain_info.head {
            Block::Macro(block) => {
                BlockInfo {
                    is_macro: true,
                    short_hash: short_hash(&block_hash),
                    next_block_hash: chain_info.main_chain_successor,
                    epoch: epoch_at(block.header.block_number),

                    block_hash,
                    parent_hash: null_hash_opt(block.header.parent_hash),
                    parent_macro_hash: null_hash_opt(block.header.parent_macro_hash),
                    block_number: block.header.block_number,
                    view_number: block.header.view_number,
                    timestamp: nimiq_to_chrono(block.header.timestamp),
                    seed: block.header.seed,
                    state_root: block.header.state_root,
                    extrinsics_root: block.header.extrinsics_root,
                    transaction_root: Some(block.header.transactions_root),

                    transactions: None,
                }
            },
            Block::Micro(block) => {
                let mut transactions = Vec::new();

                if let Some(extrinsics) = block.extrinsics {
                    for tx in extrinsics.transactions {
                        transactions.push(tx.into());
                    }
                }

                BlockInfo {
                    is_macro: false,
                    short_hash: short_hash(&block_hash),
                    next_block_hash: chain_info.main_chain_successor,
                    epoch: epoch_at(block.header.block_number),

                    block_hash,
                    parent_hash: null_hash_opt(block.header.parent_hash),
                    parent_macro_hash: None,
                    block_number: block.header.block_number,
                    view_number: block.header.view_number,
                    timestamp: nimiq_to_chrono(block.header.timestamp),
                    seed: block.header.seed,
                    state_root: block.header.state_root,
                    extrinsics_root: block.header.extrinsics_root,
                    transaction_root: None,

                    transactions: Some(transactions),
                }
            },
        }
    }
}
