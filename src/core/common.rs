use std::str::FromStr;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::{cli::Provider, core::{github::Github, gitlab::Gitlab}};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Package,
    Ping,
    Push,
    #[serde(other)]
    Unknown
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package" => Ok(EventType::Package),
            "push" => Ok(EventType::Push),
            "ping" => Ok(EventType::Ping),
            _ => Ok(EventType::Unknown)
        }
    }
}

pub struct Headers {
    pub event_type: EventType,
    pub signature: String,
}

pub trait GitProvider {
    fn webhook(self, http_request: HttpRequest, req_body: String) -> HttpResponse;
}

#[post("/webhook")]
pub async fn webhook_request(data: web::Data<Provider>, http_request: HttpRequest, req_body: String) -> impl Responder {
    let proviver = data.as_ref();
    let github = Github {
        prefix: String::from("github"),
    };
    let gitlab = Gitlab {
        prefix: String::from("gitlab"),
    };
    match proviver {
        Provider::Github => github.webhook(http_request, req_body),
        Provider::Gitlab => gitlab.webhook(http_request, req_body),
        Provider::Both => HttpResponse::Accepted().body("Accepted"),
    }
}
