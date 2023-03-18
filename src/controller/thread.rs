use crate::filters;
use crate::middleware::Context;
use crate::model::{Node, Post, Thread, Ugc, User};
use actix_web::web::{Data, Path};
use actix_web::{error, get, Responder};
use askama::Template;
use scylla::Session;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub context: Context,
    pub node: Node,
    pub thread: Thread,
    pub posts: Vec<Post>,
    pub positions: HashMap<i64, i32>,
    pub ugcs: HashMap<i64, Ugc>,
    pub users: HashMap<i64, User>,
}

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_thread);
}

#[get("/threads/{thread_id}/")]
async fn view_thread(
    path: Path<i64>,
    context: Context,
    scylla: Data<Session>,
) -> actix_web::Result<impl Responder> {
    let thread_id = path.into_inner();
    let thread = match Thread::fetch(scylla.clone(), thread_id).await {
        Ok(result) => match result {
            Some(thread) => thread,
            None => return Err(error::ErrorNotFound("Thread Not Found")),
        },
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    let (node, (posts, positions)) = match tokio::join!(
        Node::fetch(scylla.clone(), thread.node_id),
        Post::fetch_thread(scylla.clone(), thread_id, 1),
    ) {
        (Ok(node), Ok(posts)) => (
            match node {
                Some(node) => node,
                None => return Err(error::ErrorNotFound("Thread Not Found")),
            },
            posts,
        ),
        (Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
    };

    let (ugcs, users) = match tokio::join!(
        Ugc::fetch_many_posts(scylla.clone(), &posts),
        User::fetch_many_post_authors(scylla.clone(), &posts),
    ) {
        (Ok(ugcs), Ok(users)) => (ugcs, users),
        (Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
    };

    thread.bump_view_count(scylla.to_owned());

    Ok(ThreadTemplate {
        context,
        node,
        thread,
        posts,
        positions,
        ugcs,
        users,
    })
}
