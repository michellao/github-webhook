use std::{process::Command, str::FromStr};
use actix_web::{http::header::{HeaderMap, HeaderValue}, post, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

struct GithubHeaders {
    event_type: EventType,
    signature: String,
}

fn gh_headers(headers: &HeaderMap) -> Option<GithubHeaders> {
    let gh_event = headers.get("X-GitHub-Event");
    if let Some(gh_signature) = headers.get("X-Hub-Signature-256") {
        let header_value_unknown = HeaderValue::from_static("Unknown");

        let event_str = gh_event.unwrap_or(&header_value_unknown).to_str().unwrap_or("unknown");
        let event_type = EventType::from_str(event_str).unwrap();
        Some(GithubHeaders { event_type, signature: gh_signature.to_str().unwrap().to_string() })
    } else {
        None
    }
}

#[post("/webhook")]
async fn gh_webhook(http_request: HttpRequest, req_body: String) -> impl Responder {
    let headers = http_request.headers();
    match gh_headers(headers) {
        Some(github_headers) => {
            let complete_signature = github_headers.signature;
            let split_signature: Vec<&str> = complete_signature.split('=').collect();
            if split_signature.len() != 2 {
                return HttpResponse::BadRequest().body("Invalid signature format");
            }
            let hash256 = split_signature[1];
            if !verify_signature(hash256.as_bytes(), req_body.as_bytes()) {
                return HttpResponse::Unauthorized().body("Invalid signature");
            }
            let event_type = github_headers.event_type;
            match event_type {
                EventType::Package => {
                    let output = Command::new("./package.sh")
                        .output()
                        .expect("Failed to execute package.sh");
                    println!("Package event received");
                    println!("{}", std::str::from_utf8(&output.stdout).unwrap());
                },
                EventType::Ping => {
                    println!("Ping event received");
                },
                _ => {
                    println!("Nothing to do unknown event");
                }
            }
        }
        None => return HttpResponse::BadRequest().body("Invalid headers"),
    }
    HttpResponse::Accepted().body("Accepted")
}

pub fn verify_signature(signature_header: &[u8] , payload_body: &[u8]) -> bool {
    let webhook_secret = std::env::var("GH_WEBHOOK_SECRET").unwrap();
    let keypair = openssl::pkey::PKey::hmac(webhook_secret.as_bytes()).unwrap();
    let mut signer = openssl::sign::Signer::new(
        openssl::hash::MessageDigest::sha256(),
        &keypair
    ).unwrap();

    signer.update(payload_body).unwrap();
    let signature = signer.sign_to_vec().unwrap();
    let signature_to_bytes = hex::decode(signature_header).unwrap();
    openssl::memcmp::eq(&signature, &signature_to_bytes)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Package,
    Ping,
    #[serde(other)]
    Unknown
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package" => Ok(EventType::Package),
            "ping" => Ok(EventType::Ping),
            _ => Ok(EventType::Unknown)
        }
    }
}
