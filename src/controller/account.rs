use crate::middleware::{Context, Flash};
use crate::view::account::RegisterTemplate;
use actix_web::web::Form;
use actix_web::{error, get, post, Error, HttpResponse, Responder};
use serde::Deserialize;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_register).service(create_user_post);
}

#[derive(Debug, Deserialize)]
pub struct RegistrationForm {
    username: String,
    email: Option<String>,
    password: String,
    password_confirm: String,
}

#[post("/register")]
pub async fn create_user_post(
    mut context: Context,
    form: Form<RegistrationForm>,
) -> impl Responder {
    if form.password != form.password_confirm {
        context.flash(Flash::ERROR, "Password fields do not match.");
    }

    RegisterTemplate { context }
}

#[get("/register")]
pub async fn view_register(context: Context) -> impl Responder {
    RegisterTemplate { context }
}
