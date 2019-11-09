use rocket::State;
use rocket::request::Form;
use rocket_contrib::templates::Template;

use crate::resource::ResourceRenderer;
use crate::albatross::{Albatross, AnyIdentifier, ParseError};


#[derive(FromForm)]
pub struct SearchParams {
    q: String,
}

#[get("/search?<params..>")]
pub fn get_search(params: Form<SearchParams>, albatross: State<Albatross>, renderer: State<ResourceRenderer>) -> Result<Template, ()> {
    match params.q.parse::<AnyIdentifier>() {
        Ok(_ident) => {
            // TODO redirect to the appropriate page
            Err(())
        },
        Err(ParseError::Unrecognized(_q)) => {
            // TODO Search other stuff for s (e.g. aliases)
            Ok(renderer.render("search-results", (), &albatross))
        },
        Err(_) => {
            Err(())
        },
    }
}
