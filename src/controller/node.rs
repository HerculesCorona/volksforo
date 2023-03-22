use crate::filters;
use crate::middleware::context::Context;
use crate::model::{Node, Thread};
use actix_web::web::{Data, Path, Redirect};
use actix_web::{error, get, Responder};
use askama::Template;
use scylla::Session;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_forum)
        .service(view_forum_index)
        .service(view_index);
}

#[derive(Template)]
#[template(path = "forum.html")]
pub struct ForumTemplate {
    pub context: Context,
    pub node: Node,
    pub threads: Vec<(Thread, i64, i64)>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub context: Context,
    pub nodes: Vec<Node>,
}

#[get("/forums/{node_id}/")]
async fn view_forum(
    context: Context,
    scylla: Data<Session>,
    path: Path<i64>,
) -> actix_web::Result<impl Responder> {
    let node_id = path.into_inner();
    let (node, threads) = match tokio::join!(
        Node::fetch(scylla.clone(), node_id),
        Thread::fetch_node_page(scylla.clone(), node_id, 1),
    ) {
        (Ok(node), Ok(threads)) => (
            match node {
                Some(node) => node,
                None => return Err(error::ErrorNotFound("Forum not found.")),
            },
            threads,
        ),
        (Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
    };

    let thread_ids: Vec<i64> = threads.iter().map(|t| t.id).collect();
    let (mut replies, mut views) = match tokio::join!(
        Thread::fetch_many_reply_count(scylla.clone(), thread_ids.to_owned()),
        Thread::fetch_many_view_count(scylla.clone(), thread_ids)
    ) {
        (Ok(replies), Ok(views)) => (replies, views),
        (Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(ForumTemplate {
        context,
        node,
        threads: threads
            .into_iter()
            .map(|t| {
                let id = t.id;
                (
                    t,
                    replies.remove(&id).unwrap_or(0),
                    views.remove(&id).unwrap_or(0),
                )
            })
            .collect(),
    })
}

#[get("/forums/")]
async fn view_forum_index() -> impl Responder {
    Redirect::to("/").see_other()
}

#[get("/")]
async fn view_index(context: Context, scylla: Data<Session>) -> impl Responder {
    let nodes = Node::fetch_all(scylla).await.unwrap();

    IndexTemplate { context, nodes }
}
