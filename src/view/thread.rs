use crate::middleware::context::Context;
use crate::model::{Node, Post, Thread, Ugc, User};
use askama::Template;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub context: Context,
    pub node: Node,
    pub thread: Thread,
    pub posts: Vec<Post>,
    pub ugc: HashMap<Uuid, Ugc>,
    pub users: HashMap<i64, User>,
}
