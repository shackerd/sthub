use actix_files::NamedFile;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use std::path::PathBuf;
mod environment;

async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn doc_index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    // root directory is not the current working directory
    // so we need to specify the path correctly
    let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut path = PathBuf::from("./assets/");
    path.push(req_path);
    Ok(NamedFile::open(path)?)
}

async fn env_api() -> impl Responder {
    let environment = environment::JsonEnvironmentVarsTree::new("STHUB__");
    let tree = environment.build();
    web::Json(tree)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/env", web::get().to(env_api))
            .route("/doc/{filename:.*}", web::get().to(doc_index))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
