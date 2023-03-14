pub mod asset;
pub mod error;
pub mod forum;
pub mod thread;

/// Configures the web app by adding services from each web file.
///
/// @see https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#method.configure
pub fn configure(conf: &mut actix_web::web::ServiceConfig) {
    // Descending order. Order is important.
    // Route resolution will stop at the first match.
    asset::configure(conf);
}
