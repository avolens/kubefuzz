use super::K8sResourceSpec;
use crate::generator::rand::{
    gen_printable_string, gen_range, rand_i64, rand_str_regex, rand_u64, shuffle,
};

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicU64, Ordering};

static GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);

// todo: add intXX to format parsing

fn gen_ip() -> serde_json::Value {
    let mut ip = String::new();
    for _ in 0..4 {
        ip.push_str(&gen_range(0, 256).to_string());
        ip.push('.');
    }
    ip.pop();
    return serde_json::Value::String(ip);
}

// todo : add support for date format and get correct quantities to be generated
fn gen_string(propname: &str, format: &Option<String>) -> serde_json::Value {
    let lower = propname.to_lowercase();
    match lower {
        _ if lower.contains("port") => gen_range(0, 65535).to_string().into(),
        _ if lower.contains("ip") => gen_ip(),
        _ if lower == "host" => gen_ip(),
        _ if lower.contains("group") || lower == "runasuser" || lower.contains("username") => {
            gen_printable_string(
                gen_range(1, 25),
                Some("_-abcdefghijklmnopqrstuvwxyz".as_bytes()), // losely based on unix usernme naming rules
            )
            .into()
        }
        // quantities
        _ if lower == "cpu" || lower == "memory" => {
            let mut s = gen_range(1, 100).to_string();
            s.push_str("m");
            s.into()
        }
        _ => {
            if format.is_some() {
                match format.as_ref().unwrap().as_str() {
                    "date-time" => serde_json::Value::String("2020-01-01T00:00:00Z".into()),
                    "int-or-string" => rand_u64().into(),
                    &_ => gen_printable_string(gen_range(1, 25), None).into(),
                }
            } else {
                gen_printable_string(gen_range(1, 25), None).into()
            }
        }
    }
}

fn gen_bool() -> serde_json::Value {
    serde_json::Value::Bool(gen_range(0, 2) == 1)
}

fn gen_int(propname: &str) -> serde_json::Value {
    let lower = propname.to_lowercase();
    match lower {
        _ if lower.contains("port") => gen_range(0, 65535).into(),
        _ if lower.contains("group") || lower == "runasuser" || lower.contains("username") => {
            gen_range(0, 2147483647).into()
        }
        _ => serde_json::Value::Number(rand_i64().into()),
    }
}

fn gen_array(spec: &K8sResourceSpec, propname: &str) -> serde_json::Value {
    assert!(spec.items.is_some());
    let items = spec.items.as_ref().unwrap();
    let mut arr = serde_json::Value::Array(vec![]);
    debug!("generating array");

    let (min, max) = match spec.minmax {
        Some(minmax) => minmax,
        None => (1, 20),
    };

    for _ in 0..gen_range(min, max + 1) + 1 {
        arr.as_array_mut()
            .unwrap()
            .push(match items._type.as_str() {
                "string" => gen_string(propname, &items.format),
                "boolean" => gen_bool(),
                "integer" => gen_int(propname),
                "object" => gen_property(&items, propname),
                "array" => panic!("nested arrays not supported"),
                &_ => panic!("schema type not known"),
            });
    }
    return arr;
}

pub fn gen_property(spec: &K8sResourceSpec, propname: &str) -> serde_json::Value {
    // first check values and regex_values

    if !spec._enum.is_empty() && !spec._enum_regex.is_empty() {
        match gen_range(0, 2) {
            0 => {
                return spec._enum[gen_range(0, spec._enum.len())].clone();
            }
            1 => {
                return rand_str_regex(&spec._enum_regex[gen_range(0, spec._enum_regex.len())])
                    .into();
            }

            _ => {}
        }
    }

    if !spec._enum.is_empty() {
        return spec._enum[gen_range(0, spec._enum.len())].clone();
    }

    if !spec._enum_regex.is_empty() {
        return rand_str_regex(&spec._enum_regex[gen_range(0, spec._enum_regex.len())]).into();
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

        if !optionalprops.is_empty() {
            shuffle(&mut optionalprops);

            let num_optional = gen_range(
                if to_generate.is_empty() { 1 } else { 0 }, // if we have no required props, we need at least one optional
                optionalprops.len() + 1,
            );

            for opt in optionalprops[0..num_optional].to_vec() {
                to_generate.push(opt);
            }
        }

        for prop in to_generate {
            debug!("generating object {:?}", prop);
            ret[prop.as_str()] = gen_property(&spec.properties[prop.as_str()], &prop);
        }
        return ret;
    }

    // else, we have a primitive type
    match spec._type.as_str() {
        "string" => return gen_string(propname, &spec.format),
        "boolean" => return gen_bool(),
        "array" => return gen_array(spec, propname),
        "integer" => return gen_int(propname),

        &_ => panic!("type not covered"),
    }
}

pub fn gen_resource(spec: &K8sResourceSpec) -> serde_json::Value {
    let mut resc = gen_property(spec, "$");

    // lastly we make sure that version, kind and name are correctly set
    // as they are required by the API

    let gvkv = spec.gvk.as_ref().unwrap().split('.').collect::<Vec<&str>>();
    resc["apiVersion"] = format!("{}/{}", gvkv[0], gvkv[1]).into();
    resc["kind"] = gvkv[2].into();

    resc["metadata"]["name"] = format!(
        "kubefuzz-{}",
        GENERATION_COUNT.fetch_add(1, Ordering::SeqCst)
    )
    .into();

    return resc;
}
