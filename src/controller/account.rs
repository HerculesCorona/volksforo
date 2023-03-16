use crate::middleware::{Context, Flash};
use actix_web::web::Form;
use actix_web::{get, post, Responder};
use askama::Template;
use serde::Deserialize;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_register).service(put_register);
}

#[derive(Template)]
#[template(path = "account/register.html")]
pub struct RegisterTemplate {
    pub context: Context,
    pub form: RegisterForm,
}

#[derive(Debug, Deserialize, Default)]
pub struct RegisterForm {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
    password_confirm: Option<String>,
}

#[post("/register/")]
pub async fn put_register(mut context: Context, mut form: Form<RegisterForm>) -> impl Responder {
    if form.username.is_none() {
        context.flash(Flash::ERROR, "A uername is mandatory.");
    } else if form.password.is_none() {
        context.flash(Flash::ERROR, "a password is mandatory.");
    } else if form.password != form.password_confirm {
        context.flash(Flash::ERROR, "Password fields do not match.");
    }

    form.0.password = None;
    form.0.password_confirm = None;

    RegisterTemplate {
        context,
        form: form.0,
    }
}

#[get("/register/")]
pub async fn view_register(context: Context) -> impl Responder {
    RegisterTemplate {
        context,
        form: Default::default(),
    }
}
