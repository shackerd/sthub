use crate::core::configuration::{Configuration, load_configuration};
use actix_files::NamedFile;
use actix_rewrite::Engine;
use actix_web::{App, HttpRequest, HttpServer, Responder, get, web};
use std::path::PathBuf;
mod core;
mod environment;

#[get("/{filename:.*}")]
async fn index(req: HttpRequest, conf: web::Data<Configuration>) -> actix_web::Result<NamedFile> {
    let base = conf.hubs.clone().unwrap()._static.unwrap().path.unwrap();
    let mut path_builder = PathBuf::from(base);
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    path_builder.push(path);
    println!("{:#?}", path_builder);
    Ok(NamedFile::open(path_builder)?)
}

async fn env_api(conf: web::Data<Configuration>) -> impl Responder {
    let prefix = conf
        .hubs
        .clone()
        .unwrap()
        .configuration
        .unwrap()
        .providers
        .unwrap()
        .env
        .unwrap()
        .prefix
        .unwrap();

    let environment = environment::JsonEnvironmentVarsTree::new(&format!("{}__", &prefix));
    let tree = environment.build(); // this should be stored in volatile memory cache for performance purposes
    web::Json(tree)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = load_configuration("./config.yaml").await;
    let conf = conf.unwrap();
    let port = conf.network.clone().unwrap().port.unwrap();

    let mut engine = Engine::new();

    let rules = conf
        .clone()
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
            .app_data(web::Data::new(conf.clone()))
            .configure(|cfg: &mut web::ServiceConfig| config(cfg, rules.is_empty()))
            .wrap(engine.clone().middleware());
        return app;
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

fn config(cfg: &mut web::ServiceConfig, rewrite_on: bool) {
    cfg.route("/env", web::get().to(env_api));
    if !rewrite_on {
        cfg.service(index);
    }
}
