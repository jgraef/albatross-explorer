use std::io::Cursor;

use serde::Serialize;
use chrono::{DateTime, Utc};
use rocket_contrib::templates::Template;
use rocket::response::{Content, Stream};
use rocket::State;
use rocket::http::ContentType;

use beserial::Serialize as BeSerialize;
use nimiq_hash::Blake2bHash;
use nimiq_block_albatross::Block;
use nimiq_blockchain_albatross::chain_info::ChainInfo;
use nimiq_bls::bls12_381::CompressedSignature;
use nimiq_primitives::policy::epoch_at;
use nimiq_bls::bls12_381::CompressedPublicKey as BlsPublicKey;

use crate::utils::{serialize_with_format, serialize_with_format_opt, serialize_with_beserial, short_hash, null_hash_opt, nimiq_to_chrono};
use crate::resource::transaction::TransactionInfo;
use crate::resource::ResourceRenderer;
use crate::{Albatross, BlockIdentifier};



#[derive(Clone, Debug, Serialize)]
pub struct BlockInfo {
    short_hash: String,
    is_macro: bool,
    epoch: u32,

    #[serde(serialize_with = "serialize_with_format")]
    block_hash: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format_opt")]
    parent_hash: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_format_opt")]
    parent_macro_hash: Option<Blake2bHash>,

    block_number: u32,
    view_number: u32,

    timestamp: DateTime<Utc>,

    #[serde(serialize_with = "serialize_with_beserial")]
    seed: CompressedSignature,

    #[serde(serialize_with = "serialize_with_format")]
    state_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format")]
    extrinsics_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format_opt")]
    transaction_root: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_format_opt")]
    next_block_hash: Option<Blake2bHash>,

    transactions: Option<Vec<TransactionInfo>>,
}

#[derive(Clone, Debug)]
pub struct BlockMeta {
    block_producer_slot: u16,
    block_producer_num: u16,
    block_producer_pubkey: BlsPublicKey,
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

#[get("/block/<ident>")]
pub fn get_block(ident: BlockIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Template> {
    let block_info = albatross.get_block_info(&ident)?;
    Some(renderer.render("block", block_info, &albatross))
}

#[get("/block/<ident>/download")]
pub fn download_block(ident: BlockIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Option<Content<Stream<Cursor<Vec<u8>>>>> {
    let chain_info = albatross.get_chain_info(&ident)?;
    let mut buf = Vec::with_capacity(chain_info.serialized_size());
    chain_info.serialize(&mut buf).expect("Failed to serialize block");
    Some(Content(ContentType::Binary, Stream::from(Cursor::new(buf))))
}