mod core;
mod cli;

use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use crate::core::common::webhook_request;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = crate::cli::parse();
    if let Err(_) = dotenvy::dotenv() {
        let provider = &cli.provider;
        match provider {
            &cli::Provider::Github => std::env::var("GH_WEBHOOK_SECRET").expect("GH_WEBHOOK_SECRET must be set"),
            &cli::Provider::Gitlab => std::env::var("GL_WEBHOOK_SECRET").expect("GL_WEBHOOK_SECRET must be set"),
            &cli::Provider::Both => {
                std::env::var("GH_WEBHOOK_SECRET").expect("GH_WEBHOOK_SECRET must be set");
                std::env::var("GL_WEBHOOK_SECRET").expect("GL_WEBHOOK_SECRET must be set")
            },
        };
    };
    let server_host = std::env::var("SERVER_HOST").unwrap_or(String::from("localhost"));
    let server_port = std::env::var("SERVER_PORT").unwrap_or(String::from("8080"));

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cli.provider))
            .service(webhook_request)
    });
    let tls = &cli.tls;
    if let Some(config) = tls {
        let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(&config.private_key, SslFiletype::PEM)
            .expect("Private key file unset");
        builder
            .set_certificate_chain_file(&config.fullchain_key)
            .expect("Certificate chain file unset");
        println!("Started TLS Server on {}:{}", server_host, server_port);
        http_server.bind_openssl((server_host, server_port.parse().unwrap()), builder)?
            .run()
            .await
    } else {
        println!("Started HTTP Server on {}:{}", server_host, server_port);
        http_server.bind((server_host, server_port.parse().unwrap()))?
            .run()
            .await
    }
}
