use crate::middleware::Context;
use actix_files::NamedFile;
use actix_web::{error, get, post, Error, HttpRequest, Responder, Result};
use askama::Template;
use std::path::PathBuf;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(get_attachment_debug_form)
        .service(put_attachment)
        .service(view_public_file);
}

#[derive(Template)]
#[template(path = "attachment/upload.html")]
pub struct AttachmentDebugTemplate {
    pub context: Context,
}

#[get("/attachments/upload")]
async fn get_attachment_debug_form() -> Result<impl Responder> {
    Ok("OK")
}

#[post("/attachments/upload")]
async fn put_attachment() -> Result<impl Responder> {
    Ok("OK")
}

/// Dynamically access public files through the webserver.
#[get("/public/assets/{filename}")]
async fn view_public_file(req: HttpRequest) -> Result<NamedFile, Error> {
    let mut path: PathBuf = PathBuf::from("public/assets/");
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(req_path.file_name().unwrap());

    match NamedFile::open(path) {
        Ok(file) => Ok(file.use_last_modified(true)),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Err(error::ErrorNotFound("File Not Found")),
            _ => Err(error::ErrorInternalServerError(
                "Unexpected error trying to read file.",
            )),
        },
    }
}
