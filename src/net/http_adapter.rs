use actix_files::Files;
use actix_rewrite::Engine;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};

use crate::{
    core::configuration::{Configuration, try_get_rules},
    environment,
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
                .configure(|cfg: &mut web::ServiceConfig| config(cfg, rules.as_ref().is_some()))
                .wrap(engine.clone().middleware())
        })
        .bind(format!("127.0.0.1:{port}"))?
        .run()
        .await
    }
}

fn config(cfg: &mut web::ServiceConfig, _rewrite_on: bool) {
    cfg.route("/env", web::get().to(configuration_hub));

    // Always serve static files with index.html as default
    // Get static path from configuration if needed
    let static_path = "/var/www/html/"; // Or get from your configuration
    cfg.service(
        Files::new("/", static_path)
            .index_file("index.html")
            .use_last_modified(true)
            .prefer_utf8(true),
    );
}

async fn configuration_hub(_: HttpRequest, conf: web::Data<Configuration>) -> impl Responder {
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
