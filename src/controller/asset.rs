use actix_files as fs;
use actix_web::http::{header, header::ContentEncoding, StatusCode};
use actix_web::{get, Error, HttpRequest, HttpResponse, Responder};
use std::path::PathBuf;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_public_file);
}

/// Dynamically access public files through the webserver.
#[get("/public/assets/{filename:.*}")]
async fn view_public_file(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let mut path: PathBuf = PathBuf::from("public/assets/");
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    path.push(req_path.file_name().unwrap());

    let file = fs::NamedFile::open(path)?;

    Ok(file.use_last_modified(true))
}
