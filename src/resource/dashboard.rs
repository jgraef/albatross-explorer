use serde::Serialize;

use rocket_contrib::templates::Template;
use rocket::State;

use crate::albatross::Albatross;
use crate::resource::ResourceRenderer;
use crate::resource::block::BlockInfo;


#[derive(Clone, Debug, Serialize)]
pub struct DashboardInfo {
    head: BlockInfo,
    num_peers: usize,
}

#[get("/")]
pub fn get_dashboard(albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Template, ()> {
    let head = albatross.get_head_info();
    let info = DashboardInfo {
        head,
        num_peers: 0,
    };
    Ok(renderer.render("dashboard", info, &albatross))
}
