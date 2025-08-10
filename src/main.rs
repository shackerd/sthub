use actix_files::NamedFile;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

fn build_tree(vars: Vec<(String, String)>, prefix: &str) -> Value {
    let mut root = BTreeMap::new();

    for (key, value) in vars {
        if let Some(stripped) = key.strip_prefix(prefix) {
            let parts: Vec<&str> = stripped.split("__").collect();
            insert_nested(&mut root, &parts, value);
        }
    }
    serde_json::to_value(root).unwrap()
}

fn insert_nested(map: &mut BTreeMap<String, Value>, parts: &[&str], value: String) {
    if let Some((first, rest)) = parts.split_first() {
        if rest.is_empty() {
            map.insert(first.to_string(), Value::String(value));
        } else {
            let entry = map
                .entry(first.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            if let Value::Object(submap) = entry {
                let mut nested: BTreeMap<String, Value> = submap
                    .iter_mut()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                insert_nested(&mut nested, rest, value);

                let mut converted = nested
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect::<Map<String, Value>>();

                submap.append(&mut converted);
            }
        }
    }
}

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
    let vars: Vec<(String, String)> = env::vars().collect();
    let tree = build_tree(vars, "MYAPI__");
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
