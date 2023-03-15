use crate::middleware::Context;
use crate::model::{Node, Post, Thread, Ugc, User};
use actix_web::web::{Data, Path};
use actix_web::{error, get, Responder};
use scylla::Session;

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

    let (node, posts) = match tokio::join!(
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

    let (ugc, users) = match tokio::join!(
        Ugc::fetch_many_posts(scylla.clone(), &posts),
        User::fetch_many_post_authors(scylla.clone(), &posts),
    ) {
        (Ok(ugc), Ok(users)) => (ugc, users),
        (Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(crate::view::ThreadTemplate {
        context,
        node,
        thread,
        posts,
        ugc,
        users,
    })
}
