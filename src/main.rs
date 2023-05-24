#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log as rust_log;

use generator::gen::gen_resource;
use generator::load_constrained_spec;
mod conf;
mod generator;
mod log;

fn main() {
    log::initlog();
    generator::rand::seedrand();
    let slim_constraint = load_constrained_spec("constraint.yaml", "pod");

    let resc = gen_resource(&slim_constraint);

    println!("{}", serde_json::to_string_pretty(&resc).unwrap());
}
