use std::path::PathBuf;
use structopt::{clap::ArgGroup, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("token").required(true))]
pub struct Token {
    #[structopt(long="token-path", group = "token")]
    pub path: Option<PathBuf>,
    #[structopt(long="insecure-no-token", group = "token")]
    pub none: bool,
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub token: Token,
}

pub fn parse() -> Args {
    Args::from_args()
}
