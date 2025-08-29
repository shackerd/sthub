mod core;
mod environment;
mod kernel;
mod net;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let krn = kernel::boot().await.unwrap();
    // let logger = krn.setup_logger();
    // logger.splash();
    let adapter = krn.setup_http_adapter();
    adapter.run().await
}
