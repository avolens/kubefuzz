#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log as rust_log;

use mutator::{load_constrained_spec, loadspec};

mod conf;
mod log;
mod mutator;

fn main() {
    log::initlog();
    let slim_constraint = load_constrained_spec("constraint.json", "pod");

    println!("slim constraint: {:#?}", slim_constraint);
}
