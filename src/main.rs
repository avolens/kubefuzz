#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use conf::ConstraintConfig;
use jsonpath_lib as jsonpath;
use mutator::loadspec;

mod conf;
mod mutator;

fn main() {
    pretty_env_logger::init();

    // read file test.json into variable s
    let s = std::fs::read_to_string("test.json").unwrap();

    let cnfg: ConstraintConfig = serde_json::from_str(&s).unwrap();

    println!("{:?}", cnfg);
}
