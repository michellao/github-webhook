use std::{io::Write, process::{Command, Stdio}, str::FromStr};
use actix_web::{http::header::{HeaderMap, HeaderValue}, HttpRequest, HttpResponse};
use super::common::{EventType, GitProvider, Headers};

pub struct Gitlab {
    pub prefix: String,
}

fn gitlab_headers(headers: &HeaderMap) -> Option<Headers> {
    let gitlab_event = headers.get("X-Gitlab-Event");
    if let Some(gitlab_signature) = headers.get("X-Gitlab-Token") {
        let header_value_unknown = HeaderValue::from_static("Unknown");

        let event_str = gitlab_event.unwrap_or(&header_value_unknown).to_str().unwrap_or("unknown");
        let event_type = EventType::from_str(event_str).unwrap();
        Some(Headers { event_type, signature: gitlab_signature.to_str().unwrap().to_string() })
    } else {
        None
    }
}

impl GitProvider for Gitlab {
    fn webhook(self, http_request: HttpRequest, req_body: String) -> HttpResponse {
        let headers = http_request.headers();
        match gitlab_headers(headers) {
            Some(gitlab_headers) => {
                let complete_signature = gitlab_headers.signature;
                if !Gitlab::verify_signature(&complete_signature) {
                    return HttpResponse::Unauthorized().body("Invalid signature");
                }
                let event_type = gitlab_headers.event_type;
                tokio::spawn(async move {
                    match event_type {
                        EventType::Package | EventType::Push => {
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

impl Gitlab {
    fn verify_signature(secret_header: &str) -> bool {
        let webhook_secret = std::env::var("GL_WEBHOOK_SECRET").unwrap();
        webhook_secret == secret_header
    }
}
