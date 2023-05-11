#![allow(dead_code, unused)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use conf::ConstraintConfig;
use jsonpath_lib as jsonpath;
use mutator::{apply_constaintfile, loadspec};

mod conf;
mod mutator;

fn main() {
    pretty_env_logger::init();

    let slim_constraint = apply_constaintfile("constraint.json", loadspec("pod".to_string()));
}
