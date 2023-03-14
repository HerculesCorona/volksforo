use crate::middleware::context::Context;
use crate::model::Node;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub context: Context,
    pub nodes: Vec<Node>,
}
