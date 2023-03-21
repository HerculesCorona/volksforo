use crate::filters;
use crate::middleware::Context;
use crate::model::{Node, Post, Thread, Ugc, User};
use crate::util::{Paginator, PaginatorToHtml};
use actix_web::web::{Data, Form, Path};
use actix_web::{error, get, post, Responder};
use askama::Template;
use scylla::Session;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct ReplyForm {
    post: Option<String>,
}

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
    pub paginator: Paginator,
}

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_thread);
}

// TODO: Dynamic page sizing.
pub const POSTS_PER_PAGE: i64 = 20;

/// Returns which human-readable page number this position will appear in.
pub fn get_page_for_pos(pos: i64) -> i64 {
    ((std::cmp::max(1, pos) - 1) / POSTS_PER_PAGE) + 1
}

pub fn get_pages_in_thread(cnt: i64) -> i64 {
    ((std::cmp::max(1, cnt) - 1) / POSTS_PER_PAGE) + 1
}

async fn render_thread_page(
    context: Context,
    scylla: Data<Session>,
    thread_id: i64,
    page: i64,
) -> actix_web::Result<impl Responder> {
    let thread = match Thread::fetch(scylla.clone(), thread_id).await {
        Ok(result) => match result {
            Some(thread) => thread,
            None => return Err(error::ErrorNotFound("Thread Not Found")),
        },
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    let (node, (posts, positions), reply_count) = match tokio::join!(
        Node::fetch(scylla.clone(), thread.node_id),
        Post::fetch_thread(scylla.clone(), thread_id, 1),
        Thread::fetch_reply_count(scylla.clone(), thread_id),
    ) {
        (Ok(node), Ok(posts), Ok(reply_count)) => (
            match node {
                Some(node) => node,
                None => return Err(error::ErrorNotFound("Thread Not Found")),
            },
            posts,
            match reply_count {
                Some(reply_count) => reply_count,
                None => 0,
            },
        ),
        (Ok(_), Err(err), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_), Ok(_)) => return Err(error::ErrorInternalServerError(err)),
        (Ok(_), Ok(_), Err(err)) => return Err(error::ErrorInternalServerError(err)),
        (Ok(_), Err(err), Err(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Ok(_), Err(_)) => return Err(error::ErrorInternalServerError(err)),
        (Err(err), Err(_), Err(_)) => return Err(error::ErrorInternalServerError(err)),
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
        paginator: Paginator {
            base_url: format!("/threads/{}/", thread_id),
            this_page: page,
            page_count: get_pages_in_thread(reply_count),
        },
        node,
        thread,
        posts,
        positions,
        ugcs,
        users,
    })
}

//#[post("/threads/{thread_id}/post-reply")]
//async fn put_reply(
//    path: Path<i64>,
//    context: Context,
//    scylla: Data<Session>,
//    form: Form<ReplyForm>,
//) -> actix_web::Result<impl Responder> {
//    let thread = match Thread::fetch(scylla.clone(), thread_id).await {
//        Ok(result) => match result {
//            Some(thread) => thread,
//            None => return Err(error::ErrorNotFound("Thread Not Found")),
//        },
//        Err(err) => return Err(error::ErrorInternalServerError(err)),
//    };
//
//
//}

#[get("/threads/{thread_id}/")]
async fn view_thread(
    path: Path<i64>,
    context: Context,
    scylla: Data<Session>,
) -> actix_web::Result<impl Responder> {
    let thread_id = path.into_inner();
    render_thread_page(context, scylla, thread_id, 1).await
}

#[get("/threads/{thread_id}/page-{page}")]
async fn view_thread_page(
    path: Path<(i64, i64)>,
    context: Context,
    scylla: Data<Session>,
) -> actix_web::Result<impl Responder> {
    let (thread_id, page) = path.into_inner();
    render_thread_page(context, scylla, thread_id, page).await
}
