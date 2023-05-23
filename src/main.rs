#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log as rust_log;

use mutator::gen::gen_resource;
use mutator::load_constrained_spec;
mod conf;
mod log;
mod mutator;

fn main() {
    log::initlog();
    mutator::rand::seedrand();
    let slim_constraint = load_constrained_spec("constraint.yaml", "pod");

    let resc = gen_resource(&slim_constraint);

    println!("{}", serde_json::to_string_pretty(&resc).unwrap());
}
