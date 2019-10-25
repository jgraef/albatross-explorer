use serde::Serialize;

use rocket_contrib::templates::Template;
use rocket::State;
use rocket::request::{FromForm, Form};

use crate::albatross::Albatross;
use crate::resources::ResourceRenderer;
use crate::resources::block::BlockInfo;


#[derive(Clone, Debug, Serialize)]
pub struct DashBoardInfo {
    pub latest_blocks: Vec<BlockInfo>,
    num_blocks: usize,
    more_blocks: usize,
}

#[derive(FromForm)]
pub struct DashboardParams {
    num_blocks: Option<usize>,
}

#[get("/?<params..>")]
pub fn get_dashboard(params: Form<DashboardParams>, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Template, ()> {
    let num_blocks = params.num_blocks.unwrap_or(10);
    let latest_blocks = albatross.get_latest_blocks(num_blocks)?;
    Ok(renderer.render("dashboard", DashBoardInfo {
        latest_blocks,
        num_blocks,
        more_blocks: num_blocks + 10,
    }))
}
