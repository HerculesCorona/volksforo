use crate::middleware::{Context, Flash};
use crate::model::User;
use actix_web::cookie::Cookie;
use actix_web::web::{Data, Form};
use actix_web::{error, get, post, HttpRequest, Responder};
use askama::Template;
use scylla::Session;
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
pub async fn put_register(
    req: HttpRequest,
    scylla: Data<Session>,
    mut context: Context,
    form: Form<RegisterForm>,
) -> actix_web::Result<impl Responder> {
    let mut valid = true;

    let RegisterForm {
        username,
        email,
        password,
        password_confirm,
    } = form.0;

    if username.is_none() {
        valid = false;
        context.jar.flash(Flash::ERROR, "A uername is mandatory.");
    } else if password.is_none() {
        valid = false;
        context.jar.flash(Flash::ERROR, "a password is mandatory.");
    } else if password != password_confirm {
        valid = false;
        context
            .jar
            .flash(Flash::ERROR, "Password fields do not match.");
    }

    if valid {
        let user = User::create(
            scylla.to_owned(),
            username.to_owned().unwrap(),
            email.to_owned(),
            password.to_owned().unwrap(),
        )
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

        let session_token = user
            .create_session(scylla)
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

        let mut http_resp = super::GenericTemplate {
            context,
            title: "Registration Complete",
            body: "Account has been succesfully registered.",
        }
        .respond_to(&req);

        let session_cookie = Cookie::build("vf_session", session_token.to_string())
            //.domain("www.rust-lang.org")
            .path("/")
            //.secure(true)
            .http_only(true)
            .finish();

        http_resp.add_cookie(&session_cookie)?;

        Ok(http_resp)
    } else {
        Ok(RegisterTemplate {
            context,
            form: RegisterForm {
                username,
                email,
                password: None,
                password_confirm: None,
            },
        }
        .respond_to(&req))
    }
}

#[get("/register/")]
pub async fn view_register(context: Context) -> impl Responder {
    RegisterTemplate {
        context,
        form: Default::default(),
    }
}
