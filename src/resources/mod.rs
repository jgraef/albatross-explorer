pub mod genesis;
pub mod block;
pub mod error;
pub mod dashboard;
pub mod transaction;

use std::borrow::Cow;

use serde::Serialize;
use rocket_contrib::templates::Template;



#[derive(Clone, Debug, Serialize)]
pub struct ResourceContext<C: Serialize> {
    content: C,
    base: String,
    search_placeholder: String,
}

#[derive(Debug)]
pub struct ResourceRenderer {
    base_template: String,
}

impl ResourceRenderer {
    pub fn new<S: AsRef<str>>(base_template: S) -> Self {
        Self {
            base_template: base_template.as_ref().to_string(),
        }
    }

    pub fn render<S: Into<Cow<'static, str>>, C: Serialize>(&self, name: S, content: C) -> Template {
        Template::render(name, ResourceContext {
            content,
            search_placeholder: "4ed9cccd4427d67cf7d78c77e20945522acec086c8d71d4a77df34a8d7901b7b".to_string(),
            base: self.base_template.clone(),
        })
    }
}

impl Default for ResourceRenderer {
    fn default() -> Self {
        ResourceRenderer::new("base")
    }
}

