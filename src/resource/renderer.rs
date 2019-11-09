use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Cursor;

use serde::Serialize;
use rocket_contrib::templates::Template;
use rocket::response::{Content, Stream};
use rocket::http::ContentType;

use beserial::Serialize as BeSerialize;

use crate::albatross::Albatross;


pub type Download = Content<Stream<Cursor<Vec<u8>>>>;

#[derive(Clone, Debug, Serialize)]
pub struct ResourceContext<C: Serialize> {
    content: C,
    base: String,
    search_placeholder: String,
    network_name: String,
    block_number: u32,
    debug_content: Option<String>,
}

#[derive(Debug)]
pub struct ResourceRenderer {
    base_template: String,
    send_debug_content: bool,
}

impl ResourceRenderer {
    pub fn new<S: AsRef<str>>(base_template: S) -> Self {
        Self {
            base_template: base_template.as_ref().to_string(),
            send_debug_content: false,
        }
    }

    pub fn render<S: Into<Cow<'static, str>>, C: Serialize + Debug>(&self, name: S, content: C, albatross: &Albatross) -> Template {
        // TODO: If we do the rendering as part of a Guard, the handlers only have to fetch the data and
        // return a `Render<S: Into<Cow<'static, str>>, C: Serialize> { page: S, content: C, albatross: &'a Albatross }`
        // Then the guard can also look into the Request to get the `format` GET parameter to determine if we want normal
        // rendered page output or e.g. JSON.
        // The download case is special, because we don't add any extra information to the data structure.
        // But we can make a `Responder` that works with anything that is beserial::Serialize

        let debug_content = if self.send_debug_content {
            Some(format!("{:#?}", content))
        }
        else { None };

        Template::render(name, ResourceContext {
            content,
            search_placeholder: "4ed9cccd4427d67cf7d78c77e20945522acec086c8d71d4a77df34a8d7901b7b".to_string(),
            base: self.base_template.clone(),
            network_name: format!("{}", albatross.network_id()),
            block_number: albatross.block_number(),
            debug_content,
        })
    }

    pub fn download<C: BeSerialize>(&self, content: C) -> Result<Download, beserial::SerializingError> {
        let mut buf = Vec::with_capacity(BeSerialize::serialized_size(&content));
        BeSerialize::serialize(&content, &mut buf)?;
        Ok(Content(ContentType::Binary, Stream::from(Cursor::new(buf))))
    }
}

impl Default for ResourceRenderer {
    fn default() -> Self {
        ResourceRenderer::new("base")
    }
}
