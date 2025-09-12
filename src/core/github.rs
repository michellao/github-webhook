use std::str::FromStr;
use actix_web::{http::header::HeaderMap, HttpRequest, HttpResponse};
use super::common::calling_script_shell;

use super::common::{EventType, GitProvider, Headers};

pub struct Github {
    pub prefix: String,
}

fn github_headers(headers: &HeaderMap) -> Option<Headers> {
    if let Some(gh_signature) = headers.get("X-Hub-Signature-256")
        && let Some(gh_event) = headers.get("X-GitHub-Event") {
        let event_str = gh_event.to_str().unwrap();
        match EventType::from_str(event_str) {
            Ok(e) => Some(Headers { event_type: e, signature: gh_signature.to_str().unwrap().to_string() }),
            Err(_) => None,
        }
    } else {
        None
    }
}

impl GitProvider for Github
{
    fn webhook(self, http_request: HttpRequest, req_body: String) -> HttpResponse {
        let headers = http_request.headers();
        match github_headers(headers) {
            Some(github_headers) => {
                let complete_signature = github_headers.signature;
                let split_signature: Vec<&str> = complete_signature.split('=').collect();
                if split_signature.len() != 2 {
                    return HttpResponse::BadRequest().body("Invalid signature format");
                }
                let hash256 = split_signature[1];
                if !Github::verify_signature(hash256.as_bytes(), req_body.as_bytes()) {
                    return HttpResponse::Unauthorized().body("Invalid signature");
                }
                let event_type = github_headers.event_type;
                tokio::spawn({
                    calling_script_shell(self.prefix, event_type, req_body)
                });
            }
            None => return HttpResponse::BadRequest().body("Invalid headers"),
        }
        HttpResponse::Accepted().body("Accepted")
    }
}

impl Github {
    fn verify_signature(signature_header: &[u8] , payload_body: &[u8]) -> bool {
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
}
