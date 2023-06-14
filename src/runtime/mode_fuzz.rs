use std::collections::HashMap;

use crate::args::Fuzz;
use crate::executor::{deploy_resource, get_client};
use crate::generator::k8sresc::K8sResourceSpec;
use crate::generator::{gen::gen_resource, load_constrained_spec};

struct CorpusEntry<'a> {
    pub data: serde_json::Value,
    pub constraint: &'a K8sResourceSpec,
}

pub async fn run(args: &Fuzz) {
    let client = get_client(args.kubeconfig.as_deref()).await;

    let mut constraintmap = HashMap::<String, K8sResourceSpec>::new();
    let mut corpus = HashMap::<f64, CorpusEntry>::new();

    for file in args.constraints[0].split(",") {
        let cspec = load_constrained_spec(file, &args.schemadir);
        constraintmap.insert(cspec.gvk.clone().unwrap(), cspec);
    }

    // todo: sanity check namespaces
    do_fuzz_iteration(client, &mut corpus, &constraintmap).await;
    /*
    we need to implement the following logic:
    1. get the constrained specs
    2. loop:
        - generate random specs
        - throw them against api, add those who get new coverage
        - mutate existing, add those who get new coverage
    */
}

async fn do_fuzz_iteration(
    client: kube::Client,
    corpus: &mut HashMap<f64, CorpusEntry<'_>>,
    constraintmap: &HashMap<String, K8sResourceSpec>,
) {
    debug!("doing fuzzing iteration");

    let sample = gen_resource(&constraintmap.iter().next().unwrap().1);

    println!("{}", serde_json::to_string_pretty(&sample).unwrap());

    // write sample to file

    let file = std::fs::File::create("sample.json").unwrap();
    serde_json::to_writer_pretty(file, &sample).unwrap();

    let a = deploy_resource(sample, ".v1.Pod", client, "default").await;
    println!("{:?}", a);
}

fn calculate_coverage() {}
