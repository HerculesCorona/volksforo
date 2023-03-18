use crate::middleware::Context;
use actix_web::Responder;
use askama::Template;

pub mod account;
pub mod asset;
pub mod error;
pub mod node;
pub mod thread;

/// Configures the web app by adding services from each web file.
///
/// @see https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#method.configure
pub fn configure(conf: &mut actix_web::web::ServiceConfig) {
    // Descending order. Order is important.
    // Route resolution will stop at the first match.
    account::configure(conf);
    asset::configure(conf);
    node::configure(conf);
    thread::configure(conf);
}

#[derive(Template)]
#[template(path = "container/public.html")]
struct GenericTemplate<'a> {
    context: Context,
    title: &'a str,
    body: &'a str,
}
