mod github;
use actix_web::{App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::github::gh_webhook;

struct Config {
    private_key_file: String,
    chain_key_file: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(_) = dotenvy::dotenv() {
       std::env::var("GH_WEBHOOK_SECRET").expect("GH_WEBHOOK_SECRET must be set");
    }
    let server_host = std::env::var("SERVER_HOST").unwrap_or(String::from("localhost"));
    let server_port = std::env::var("SERVER_PORT").unwrap_or(String::from("8080"));

    let http_server = HttpServer::new(|| {
        App::new()
            .service(gh_webhook)
    });

    if let Some(config) = get_config() {
        let mut builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(config.private_key_file, SslFiletype::PEM)
            .expect("Private key file unset");
        builder
            .set_certificate_chain_file(config.chain_key_file)
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

fn get_config() -> Option<Config> {
    let mut args = std::env::args();
    if let Some(_) = args.position(|arg| arg == "--private-key") {
        let private_value_file = args.next().expect("");
        if let Some(_) = args.position(|arg| arg == "--fullchain-key") {
            let chain_value_file = args.next().expect("Fullchain key file unset");
            return Some(Config {
                private_key_file: private_value_file,
                chain_key_file: chain_value_file,
            })
        }
    }
    None
}
