use super::K8sResourceSpec;
use crate::mutator::rand::{gen_printable_string, gen_range, rand_i64};
use rand::prelude::SliceRandom;

fn gen_string() -> serde_json::Value {
    serde_json::Value::String(gen_printable_string(gen_range(1, 25)))
}

fn gen_bool() -> serde_json::Value {
    serde_json::Value::Bool(gen_range(0, 2) == 1)
}

pub fn gen_property(spec: &K8sResourceSpec) -> serde_json::Value {
    // TODO: ensure that the types always match!

    if !spec._enum.is_empty() {
        return spec._enum[gen_range(0, spec._enum.len())].clone();
    }

    if spec._type == "object" {
        let mut ret = serde_json::Value::default();
        let mut to_generate: Vec<String> = vec![];

        // first go through the required values
        for req in &spec.required {
            to_generate.push(req.to_string());
        }

        // now randomly choose a subset of the other properties to generate
        let mut optionalprops: Vec<String> = spec
            .properties
            .keys()
            .filter(|x| !spec.required.contains(x))
            .cloned()
            .collect();

        // TODO: make this use our RNG
        optionalprops.shuffle(&mut rand::thread_rng());

        let num_optional = gen_range(0, optionalprops.len() + 1);

        for opt in optionalprops[0..num_optional].to_vec() {
            to_generate.push(opt);
        }

        debug!("generating object {:?}", to_generate);
        for req in to_generate {
            ret[req.as_str()] = gen_property(&spec.properties[req.as_str()]);
        }
        return ret;
    }

    // else, we have a primitive type
    match spec._type.as_str() {
        "string" => return gen_string(),
        "boolean" => return gen_bool(),
        "array" => return serde_json::Value::Array(vec![]),
        "integer" => return serde_json::Value::Number(rand_i64().into()),

        &_ => panic!("type not covered"),
    }
}

pub fn gen_resource(spec: &K8sResourceSpec) -> serde_json::Value {
    return gen_property(spec);
}
