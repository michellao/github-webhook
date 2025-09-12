use std::{io::Write, process::{Command, Stdio}, str::FromStr};
use actix_web::{http::header::{HeaderMap, HeaderValue}, HttpRequest, HttpResponse};

use super::common::{EventType, GitProvider, Headers};

pub struct Github {
    pub prefix: String,
}

fn github_headers(headers: &HeaderMap) -> Option<Headers> {
    let gh_event = headers.get("X-GitHub-Event");
    if let Some(gh_signature) = headers.get("X-Hub-Signature-256") {
        let header_value_unknown = HeaderValue::from_static("Unknown");

        let event_str = gh_event.unwrap_or(&header_value_unknown).to_str().unwrap_or("unknown");
        let event_type = EventType::from_str(event_str).unwrap();
        Some(Headers { event_type, signature: gh_signature.to_str().unwrap().to_string() })
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
                tokio::spawn(async move {
                    match event_type {
                        EventType::Package => {
                            let program_name = format!("./{}package.sh", self.prefix);
                            let mut child = Command::new(program_name)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Failed to execute package.sh");

                            let mut stdin = child.stdin.take().expect("Failed to open stdin");
                            std::thread::spawn(move || {
                                stdin.write_all(req_body.as_bytes()).expect("Failed to write to stdin");
                            });

                            let output = child.wait_with_output().expect("Failed to read stdout");
                            println!("Package event received: {}", String::from_utf8_lossy(&output.stdout));
                        },
                        EventType::Ping => {
                            println!("Ping event received");
                        },
                        _ => {
                            println!("Nothing to do unknown event");
                        }
                    }
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
