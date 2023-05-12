#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log as rust_log;

use mutator::{apply_constaintfile, loadspec};

mod conf;
mod log;
mod mutator;

fn main() {
    log::initlog();
    let slim_constraint = apply_constaintfile("constraint.json", loadspec("pod".to_string()));
}
