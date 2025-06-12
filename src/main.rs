mod github;
use actix_web::{App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::github::gh_webhook;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_) = dotenvy::dotenv() {
       std::env::var("GH_WEBHOOK_SECRET").expect("GH_WEBHOOK_SECRET must be set");
    }
    let server_host = std::env::var("SERVER_HOST").unwrap_or(String::from("localhost"));
    let server_port = std::env::var("SERVER_PORT").unwrap_or(String::from("8080"));
    let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("../key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("../fullchain.pem").unwrap();
    HttpServer::new(|| {
        App::new()
            .service(gh_webhook)
    })
    //.bind((server_host, server_port.parse().unwrap()))?
    .bind_openssl((server_host, server_port.parse().unwrap()), builder)?
    .run()
    .await
}
