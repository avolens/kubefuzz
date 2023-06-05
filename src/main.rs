#![allow(dead_code, unused)]
extern crate pretty_env_logger;

#[macro_use]
extern crate log as rust_log;

use generator::gen::gen_resource;
use generator::load_constrained_spec;
use std::fs::File;
use std::io::Write;

mod conf;
mod executor;
mod generator;
mod log;

#[tokio::main]
async fn main() {
    log::initlog();
    generator::rand::seedrand();
    let slim_constraint = load_constrained_spec("constraint.yaml", "pod");

    let resc = gen_resource(&slim_constraint);

    // write resc in yaml format to file
    let mut file = File::create("resc.yaml").unwrap();
    let yaml_value = serde_yaml::to_string(&resc).unwrap();

    file.write_all(yaml_value.as_bytes()).unwrap();

    let cl = executor::get_client("configfile").await;

    executor::deploy_resource(resc, &slim_constraint.gvk.unwrap(), cl).await;
}
