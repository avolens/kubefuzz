use std::collections::HashMap;

use crate::args::Fuzz;
use crate::executor::get_client;
use crate::generator::load_constrained_spec;
use crate::generator::K8sResourceSpec;

pub fn run(args: &Fuzz) {
    let client = get_client(args.kubeconfig.as_deref());

    let mut constraintmap = HashMap::<String, K8sResourceSpec>::new();

    let constraintfiles = args.constraints[0].split(",");

    for file in constraintfiles {
        let cspec = load_constrained_spec(file, &args.schemadir);
        constraintmap.insert(cspec.gvk.clone().unwrap(), cspec);
    }

    /*
    we need to implement the following logic:
    1. get the constrained specs
    2. loop:
        - generate random specs
        - throw them against api, add those who get new coverage
        - mutate existing, add those who get new coverage
    */
}
