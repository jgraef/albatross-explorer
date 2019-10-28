use serde::Serialize;

use rocket_contrib::templates::Template;
use rocket::State;
use rocket::request::{FromForm, Form};

use crate::albatross::Albatross;
use crate::resource::ResourceRenderer;
use crate::resource::block::BlockInfo;


#[derive(Clone, Debug, Serialize)]
pub struct BlockchainInfo {
    pub latest_blocks: Vec<BlockInfo>,
    num_blocks: usize,
    more_blocks: usize,
}

#[derive(FromForm)]
pub struct BlockchainParams {
    num_blocks: Option<usize>,
}

#[get("/blockchain-info?<params..>")]
pub fn get_blockchain(params: Form<BlockchainParams>, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Template, ()> {
    let num_blocks = params.num_blocks.unwrap_or(10);
    let latest_blocks = albatross.get_latest_blocks(num_blocks)?;
    Ok(renderer.render("blockchain", BlockchainInfo {
        latest_blocks,
        num_blocks,
        more_blocks: num_blocks + 10,
    }, &albatross))
}
