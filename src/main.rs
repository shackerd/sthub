use actix_files::NamedFile;
use actix_rewrite::Engine;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use std::path::PathBuf;

use crate::core::configuration::load_configuration;
mod core;
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
    let environment = environment::JsonEnvironmentVarsTree::new("MYAPI__");
    let tree = environment.build(); // this should be stored in volatile memory cache for performance purposes
    web::Json(tree)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = load_configuration("./config.yaml").await;

    let mut engine = Engine::new();

    // test purposes
    let rules = conf
        .unwrap()
        .hubs
        .unwrap()
        ._static
        .unwrap()
        .rewrite_rules
        .unwrap();

    engine.add_rules(&rules).expect("failed to process rules");

    HttpServer::new(move || {
        App::new()
            .wrap(engine.clone().middleware())
            .route("/env", web::get().to(env_api))
            .route("/doc/{filename:.*}", web::get().to(doc_index))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
