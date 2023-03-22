use actix_files::NamedFile;
use actix_web::{error, get, Error, HttpRequest};
use std::path::PathBuf;

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_public_file);
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
