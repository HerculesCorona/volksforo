use crate::filters;
use crate::middleware::Context;
use crate::model::{Node, Post, Thread, Ugc, User};
use crate::util::{Paginator, PaginatorToHtml};
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::web::{Data, Path, Redirect};
use actix_web::{error, get, post, HttpRequest, Responder};
use askama::Template;
use scylla::Session;
use std::collections::HashMap;

#[derive(Debug, Default, MultipartForm)]
pub struct ReplyForm {
    content: Option<Text<String>>,
}

#[derive(Template)]
#[template(path = "thread.html")]
pub struct ThreadTemplate {
    pub context: Context,
    pub node: Node,
    pub thread: Thread,
    pub posts: Vec<Post>,
    pub positions: HashMap<i64, i64>,
    pub ugcs: HashMap<i64, Ugc>,
    pub users: HashMap<i64, User>,
    pub paginator: Paginator,
}

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(put_reply)
        .service(view_thread)
        .service(view_thread_page);
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

pub async fn get_thread_or_error(
    scylla: Data<Session>,
    thread_id: &i64,
) -> actix_web::Result<Thread> {
    match Thread::fetch(scylla, thread_id).await {
        Ok(result) => match result {
            Some(thread) => Ok(thread),
            None => Err(error::ErrorNotFound("Thread Not Found")),
        },
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

async fn render_thread_page(
    context: Context,
    scylla: Data<Session>,
    thread_id: i64,
    page: i64,
) -> actix_web::Result<impl Responder> {
    let thread = get_thread_or_error(scylla.clone(), &thread_id).await?;

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
            reply_count.unwrap_or(0),
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

#[post("/threads/{thread_id}/post-reply")]
async fn put_reply(
    path: Path<i64>,
    context: Context,
    scylla: Data<Session>,
    form: MultipartForm<ReplyForm>,
) -> actix_web::Result<impl Responder> {
    let thread_id = path.into_inner();
    let thread = get_thread_or_error(scylla.clone(), &thread_id).await?;
    let ugc = Ugc::create_for_visitor(
        scylla.clone(),
        &context.visitor,
        form.content.as_ref().expect("No post").0.to_string(),
    ) // TODO: Sanitize
    .await
    .map_err(error::ErrorInternalServerError)?;
    let snowflake_id = crate::util::snowflake_id()
        .await
        .map_err(error::ErrorInternalServerError)?;
    let post = Post {
        id: snowflake_id,
        thread_id,
        created_at: ugc.created_at,
        user_id: context.visitor.user.as_ref().map(|u| u.id),
        ugc_id: ugc.id,
    };
    let pos = post
        .insert(scylla.clone())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let page = get_page_for_pos(pos);
    if page > 1 {
        Ok(Redirect::to(format!("/threads/{}/page-{}", thread.id, page)).see_other())
    } else {
        Ok(Redirect::to(format!("/threads/{}/", thread.id)).see_other())
    }
}

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
    req: HttpRequest,
    path: Path<(i64, i64)>,
    context: Context,
    scylla: Data<Session>,
) -> actix_web::Result<impl Responder> {
    let (thread_id, page) = path.into_inner();
    Ok(render_thread_page(context, scylla, thread_id, page)
        .await?
        .respond_to(&req))
}
