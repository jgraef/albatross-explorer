use serde::Serialize;
use rocket::http::Status;
use rocket::request::Request;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::albatross::Albatross;
use crate::resource::ResourceRenderer;


#[derive(Clone, Debug, Serialize)]
pub struct ErrorInfo {
    code: u16,
    name: String,
    description: String,
}

impl From<Status> for ErrorInfo {
    fn from(status: Status) -> Self {
        Self {
            code: status.code,
            name: status.reason.to_string(),
            description: "".to_string()
        }
    }
}

#[catch(404)]
pub fn not_found(request: &Request) -> Template {
    let renderer = request.guard::<State<ResourceRenderer>>().expect("Missing renderer");
    let albatross = request.guard::<State<Albatross>>().expect("Missing Albatross");
    renderer.render("error", ErrorInfo::from(Status::NotFound), &albatross)
}
