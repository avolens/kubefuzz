#![allow(dead_code, unused)]
extern crate pretty_env_logger;
extern crate serde_transcode;

#[macro_use]
extern crate log as rust_log;
use args::{Action, Arguments};
use clap::Parser;
mod runtime;

mod args;
mod conf;
mod executor;
mod generator;
mod log;
mod mutator;

#[tokio::main]
async fn main() {
    log::initlog();

    let args = Arguments::parse();

    let seed = match args.seed {
        Some(seed) => seed,
        None => generator::rand::seedrand(),
    };

    info!("running wiht seed {}", seed);

    match args.action {
        Action::Generate(args) => runtime::mode_generate::run(&args),
        Action::Mutate(args) => runtime::mode_mutate::run(&args),
        Action::Fuzz(args) => runtime::mode_fuzz::run(&args).await,
        Action::GetSchemas(args) => runtime::mode_getschemas::run(&args),
    }
}
