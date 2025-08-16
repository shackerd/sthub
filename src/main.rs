use crate::core::configuration::load_configuration;
use actix_files::NamedFile;
use actix_rewrite::Engine;
use actix_web::{App, HttpRequest, HttpServer, Responder, get, web};
use std::path::PathBuf;
mod core;
mod environment;

#[get("/{filename:.*}")]
async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
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

    let rules = conf
        .unwrap()
        .hubs
        .unwrap()
        ._static
        .unwrap()
        .rewrite_rules
        .unwrap();

    if !rules.is_empty() {
        engine.add_rules(&rules).expect("failed to process rules");
    }

    HttpServer::new(move || {
        let app = App::new()
            .configure(|cfg: &mut web::ServiceConfig| config(cfg, rules.is_empty()))
            .wrap(engine.clone().middleware());
        return app;
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn config(cfg: &mut web::ServiceConfig, rewrite_on: bool) {
    cfg.route("/env", web::get().to(env_api));
    if !rewrite_on {
        cfg.service(index);
    }
}
