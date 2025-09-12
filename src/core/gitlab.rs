use std::str::FromStr;
use actix_web::http::header::HeaderValue;
use actix_web::{http::header::HeaderMap, HttpRequest, HttpResponse};
use super::common::{EventType, GitProvider, Headers};
use super::common::calling_script_shell;

pub struct Gitlab {
    pub prefix: String,
}

fn parsing_event_gitlab(gitlab_event: &HeaderValue) -> Option<EventType> {
    let event_str = gitlab_event.to_str().unwrap();
    let search_str = ["Tag", "Push"];
    for s in search_str {
        let parsing_event = event_str.find(s);
        if let Some(_) = parsing_event {
            return Some(EventType::from_str(&s.to_lowercase()).unwrap());
        }
    }
    None
}

fn gitlab_headers(headers: &HeaderMap) -> Option<Headers> {
    if let Some(gitlab_signature) = headers.get("X-Gitlab-Token")
        && let Some(gitlab_event) = headers.get("X-Gitlab-Event") {
        match parsing_event_gitlab(gitlab_event) {
            Some(event) => Some(Headers { event_type: event, signature: gitlab_signature.to_str().unwrap().to_string() }),
            None => None
        }
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
                tokio::spawn({
                    calling_script_shell(self.prefix, event_type, req_body)
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
