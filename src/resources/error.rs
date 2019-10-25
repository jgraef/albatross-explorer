use serde::Serialize;
use rocket::http::Status;


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