use serde::Serialize;
use chrono::{DateTime, Utc};
use rocket_contrib::templates::Template;
use rocket::State;

use nimiq_hash::{Hash, Blake2bHash};
use nimiq_block_albatross::{Block, BlockHeader, MacroHeader, MicroHeader, ForkProof, ViewChangeProof};
use nimiq_blockchain_albatross::chain_info::ChainInfo;
use nimiq_bls::bls12_381::CompressedSignature;
use nimiq_primitives::policy::epoch_at;
use nimiq_collections::grouped_list::GroupedList;
use nimiq_primitives::coin::Coin;
use nimiq_keys::Address;

use crate::utils::{serialize_with_format, serialize_with_format_opt, serialize_with_beserial,
                   short_hash, null_hash_opt, nimiq_to_chrono, serialize_datetime,
                   serialize_with_hex, serialize_bitset};
use crate::resource::transaction::TransactionInfo;
use crate::resource::{ResourceRenderer, Download};
use crate::{Albatross, BlockIdentifier};
use nimiq_collections::bitset::BitSet;


#[derive(Clone, Debug, Serialize)]
pub struct BlockHeaderInfo {
    is_macro: bool,
    short_hash: String,
    epoch: u32,

    #[serde(serialize_with = "serialize_with_format")]
    block_hash: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format_opt")]
    parent_hash: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_format_opt")]
    parent_macro_hash: Option<Blake2bHash>,

    #[serde(serialize_with = "serialize_with_format_opt")]
    next_block_hash: Option<Blake2bHash>,

    block_number: u32,
    view_number: u32,

    #[serde(serialize_with = "serialize_datetime")]
    timestamp: DateTime<Utc>,

    #[serde(serialize_with = "serialize_with_beserial")]
    seed: CompressedSignature,

    #[serde(serialize_with = "serialize_with_format")]
    state_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format")]
    extrinsics_root: Blake2bHash,

    #[serde(serialize_with = "serialize_with_format_opt")]
    transaction_root: Option<Blake2bHash>,
}

impl BlockHeaderInfo {
    pub fn new_macro_header(header: MacroHeader, next_block_hash: Option<Blake2bHash>) -> Self {
        let block_hash = header.hash();
        Self {
            is_macro: true,
            short_hash: short_hash(&block_hash),
            next_block_hash,
            epoch: epoch_at(header.block_number),

            block_hash,
            parent_hash: null_hash_opt(header.parent_hash),
            parent_macro_hash: null_hash_opt(header.parent_macro_hash),
            block_number: header.block_number,
            view_number: header.view_number,
            timestamp: nimiq_to_chrono(header.timestamp),
            seed: header.seed,
            state_root: header.state_root,
            extrinsics_root: header.extrinsics_root,
            transaction_root: Some(header.transactions_root),
        }
    }

    pub fn new_micro_header(header: MicroHeader, next_block_hash: Option<Blake2bHash>) -> Self {
        let block_hash = header.hash();
        Self {
            is_macro: false,
            short_hash: short_hash(&block_hash),
            next_block_hash,
            epoch: epoch_at(header.block_number),

            block_hash,
            parent_hash: null_hash_opt(header.parent_hash),
            parent_macro_hash: None,
            block_number: header.block_number,
            view_number: header.view_number,
            timestamp: nimiq_to_chrono(header.timestamp),
            seed: header.seed,
            state_root: header.state_root,
            extrinsics_root: header.extrinsics_root,
            transaction_root: None,
        }
    }
}

impl From<BlockHeader> for BlockHeaderInfo {
    fn from(header: BlockHeader) -> Self {
        match header {
            BlockHeader::Macro(header) => Self::from(header),
            BlockHeader::Micro(header) => Self::from(header),
        }
    }
}

impl From<MacroHeader> for BlockHeaderInfo {
    fn from(macro_header: MacroHeader) -> Self {
        Self::new_macro_header(macro_header, None)
    }
}

impl From<MicroHeader> for BlockHeaderInfo {
    fn from(micro_header: MicroHeader) -> Self {
        Self::new_micro_header(micro_header, None)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ForkProofInfo {
    header1: BlockHeaderInfo,

    header2: BlockHeaderInfo,

    #[serde(serialize_with = "serialize_with_beserial")]
    justification1: CompressedSignature,

    #[serde(serialize_with = "serialize_with_beserial")]
    justification2: CompressedSignature,
}

impl From<ForkProof> for ForkProofInfo {
    fn from(fork_proof: ForkProof) -> Self {
        ForkProofInfo {
            header1: fork_proof.header1.into(),
            header2: fork_proof.header2.into(),
            justification1: fork_proof.justification1,
            justification2: fork_proof.justification2,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SlotInfo {
    #[serde(serialize_with = "serialize_with_format")]
    staker_address: Address,

    #[serde(serialize_with = "serialize_with_format")]
    reward_address: Address,

    num_slots: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct MacroBlockInfo {
    slots: Vec<SlotInfo>,
    #[serde(serialize_with = "serialize_with_format")]
    slash_fine: Coin,
    #[serde(serialize_with = "serialize_bitset")]
    slashed_set: BitSet,
}

#[derive(Clone, Debug, Serialize)]
pub struct MicroBlockInfo {
    num_transactions: usize,
    transactions: Vec<TransactionInfo>,
    #[serde(serialize_with = "serialize_with_hex")]
    extra_data: Vec<u8>,
    fork_proofs: Vec<ForkProofInfo>,

    #[serde(serialize_with = "serialize_with_beserial")]
    signature: CompressedSignature,
    view_change_proof: Option<ViewChangeProofInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ViewChangeProofInfo {
    #[serde(serialize_with = "serialize_bitset")]
    signers: BitSet,
    #[serde(serialize_with = "serialize_with_beserial")]
    signature: CompressedSignature,
}

impl From<ViewChangeProof> for ViewChangeProofInfo {
    fn from(proof: ViewChangeProof) -> Self {
        Self {
            signers: proof.signers,
            signature: proof.signature.0.compress(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockInfo {
    #[serde(flatten)]
    header: BlockHeaderInfo,

    #[serde(flatten)]
    macro_info: Option<MacroBlockInfo>,
    #[serde(flatten)]
    micro_info: Option<MicroBlockInfo>,
}

impl From<ChainInfo> for BlockInfo {
    fn from(chain_info: ChainInfo) -> Self {
        let next_block_hash = chain_info.main_chain_successor;
        let block_hash = chain_info.head.hash();

        match chain_info.head {
            Block::Macro(block) => {
                let extrinsics = block.extrinsics
                    .unwrap_or_else(|| panic!("Micro block is missing extrinsics: {}", block_hash));

                let slots = GroupedList::from(extrinsics.slot_addresses).iter_groups().cloned()
                    .map(|g| SlotInfo {
                        staker_address: g.1.staker_address,
                        reward_address: g.1.reward_address,
                        num_slots: g.0,
                    }).collect();

                BlockInfo {
                    header: BlockHeaderInfo::new_macro_header(block.header, next_block_hash),
                    macro_info: Some(MacroBlockInfo {
                        slots,
                        slash_fine: extrinsics.slash_fine,
                        slashed_set: extrinsics.slashed_set,
                    }),
                    micro_info: None,
                }
            },
            Block::Micro(block) => {
                let extrinsics = block.extrinsics
                    .unwrap_or_else(|| panic!("Micro block is missing extrinsics: {}", block_hash));

                let transactions = extrinsics.transactions.into_iter()
                    .map(TransactionInfo::from)
                    .collect::<Vec<TransactionInfo>>();

                let fork_proofs = extrinsics.fork_proofs.into_iter()
                    .map(|fork_proof| fork_proof.into())
                    .collect::<Vec<ForkProofInfo>>();

                let view_change_proof = block.justification
                    .view_change_proof
                    .map(ViewChangeProofInfo::from);

                BlockInfo {
                    header: BlockHeaderInfo::new_micro_header(block.header, next_block_hash),
                    macro_info: None,
                    micro_info: Some(MicroBlockInfo {
                        num_transactions: transactions.len(),
                        transactions,
                        extra_data: extrinsics.extra_data,
                        fork_proofs,
                        signature: block.justification.signature,
                        view_change_proof,
                    }),
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
pub fn download_block(ident: BlockIdentifier, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Download, ()> {
    let chain_info = albatross.get_chain_info(&ident).ok_or(())?;
    renderer.download(chain_info.head).map_err(|e| warn!("Download failed: {}", e))
}