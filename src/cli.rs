use std::path::PathBuf;

use clap::{Parser, Args, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub tls: Option<Tls>,

    /// Change the calling file
    #[arg(value_enum, default_value_t = Provider::Github)]
    pub provider: Provider,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Provider {
    Github,
    Gitlab,
    Both
}

#[derive(Args, Debug, Clone)]
#[group(requires_all = ["private_key", "fullchain_key"])]
pub struct Tls {
    /// Path to a private key to enable TLS
    #[arg(short, long, value_name = "FILE", required = false)]
    pub private_key: PathBuf,

    /// Path to a fullchain key to enable TLS
    #[arg(short, long, value_name = "FILE", required = false)]
    pub fullchain_key: PathBuf
}

pub fn parse() -> Cli {
    Cli::parse()
}
