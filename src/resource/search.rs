use rocket::request::{Form, FromForm};
use rocket::State;
use rocket_contrib::templates::Template;

use crate::resource::ResourceRenderer;
use crate::albatross::{Albatross, BlockIdentifier};


#[derive(FromForm)]
pub struct SearchParams {
    q: String,
}

#[get("/search?<params..>")]
pub fn get_search(params: Form<SearchParams>, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Template, ()> {
    Err(()) // TODO
}
