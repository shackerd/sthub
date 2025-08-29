use actix_files::Files;
use actix_rewrite::Engine;
use actix_web::{App, HttpServer, web};

use crate::{
    core::configuration::{Configuration, try_get_rules},
    net::environment_middleware::EnvironmentMiddleware,
};

pub struct HttpAdapter<'a> {
    configuration: &'a Configuration,
}

impl<'a> HttpAdapter<'a> {
    pub fn new(configuration: &'a Configuration) -> Self {
        Self { configuration }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let conf = self.configuration.clone();

        let mut engine = Engine::new();

        let rules = try_get_rules(self.configuration);

        if let Some(r) = rules.as_ref() {
            engine.add_rules(r).expect("failed to process rules");
        }

        let port = conf.network.clone().unwrap().port.unwrap();

        println!("running sthub on port:{port}");

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(conf.clone()))
                .wrap(engine.clone().middleware())
                .wrap(EnvironmentMiddleware)
                .configure(|cfg: &mut web::ServiceConfig| config(cfg, &conf))
        })
        .bind(format!("127.0.0.1:{port}"))?
        .run()
        .await
    }
}

fn config(cfg: &mut web::ServiceConfig, conf: &Configuration) {
    let path = conf
        .hubs
        .clone()
        .and_then(|h| h._static)
        .and_then(|f| f.path)
        .unwrap_or_else(|| "/var/www/html/".to_string());

    cfg.service(
        Files::new("/", path)
            .index_file("index.html")
            .use_last_modified(true)
            .prefer_utf8(true),
    );
}
