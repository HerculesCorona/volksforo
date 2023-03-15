use crate::middleware::context::Context;
use askama::Template;

#[derive(Template)]
#[template(path = "account/register.html")]
pub struct RegisterTemplate {
    pub context: Context,
}
