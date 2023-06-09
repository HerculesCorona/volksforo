// Documentation for middleware can be found here:
// https://actix.rs/docs/middleware/
// https://github.com/actix/actix-web/blob/master/src/middleware/normalize.rs
// https://github.com/actix/actix-extras/tree/master/actix-session/src

pub mod context;
pub use context::Context;
pub mod flash;
pub use flash::Flash;
pub use flash::FlashJar;
pub use flash::FlashMessage;
