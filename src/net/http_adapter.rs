use actix_files::Files;
use actix_rewrite::Engine;
use actix_web::{App, HttpServer, web};

use crate::{
    core::configuration::Configuration,
    net::{environment_middleware::EnvironmentMiddleware, headers_middleware::HeadersMiddleware},
};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "localhost";
const DEFAULT_STATIC_PATH: &str = "/var/www/html/";
const DEFAULT_DOCUMENT: &str = "index.html";
const DEFAULT_REMOTE_PATH: &str = "/";

pub struct HttpAdapter<'a> {
    configuration: &'a Configuration,
}

impl<'a> HttpAdapter<'a> {
    pub fn new(configuration: &'a Configuration) -> Self {
        Self { configuration }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let mut engine = Engine::new();

        let rules = self
            .configuration
            .hubs
            .as_ref()
            .and_then(|h| h._static.as_ref())
            .and_then(|s| s.rewrite_rules.clone());

        if let Some(r) = rules.as_ref() {
            engine.add_rules(r).expect("failed to process rules");
        }

        let host = self
            .configuration
            .network
            .as_ref()
            .and_then(|f| f.host.clone())
            .unwrap_or_else(|| DEFAULT_HOST.to_string());

        let port = self
            .configuration
            .network
            .as_ref()
            .and_then(|f| f.port)
            .unwrap_or(DEFAULT_PORT);

        let static_path = self
            .configuration
            .hubs
            .as_ref()
            .and_then(|h| h._static.as_ref())
            .and_then(|s| s.path.clone())
            .unwrap_or_else(|| DEFAULT_STATIC_PATH.to_string());

        let remote_path = self
            .configuration
            .hubs
            .as_ref()
            .and_then(|h| h._static.as_ref())
            .and_then(|c| c.remote_path.clone())
            .unwrap_or_else(|| DEFAULT_REMOTE_PATH.to_string());

        let conf = self.configuration.clone();

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(conf.clone()))
                .wrap(engine.clone().middleware())
                .wrap(EnvironmentMiddleware)
                .wrap(HeadersMiddleware)
                .configure(|cfg: &mut web::ServiceConfig| config(cfg, &remote_path, &static_path))
        })
        .bind(format!("{host}:{port}"))?
        .run()
        .await
    }
}

fn config(cfg: &mut web::ServiceConfig, remote_path: &str, static_path: &str) {
    cfg.service(
        Files::new(remote_path, static_path)
            .index_file(DEFAULT_DOCUMENT)
            .use_last_modified(true)
            .prefer_utf8(true),
    );
}
