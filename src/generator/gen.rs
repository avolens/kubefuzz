use super::K8sResourceSpec;
use crate::generator::rand::{
    gen_printable_string, gen_range, rand_date_time, rand_int, rand_str_regex, shuffle,
};

use std::sync::atomic::{AtomicU64, Ordering};

static GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);

fn gen_ip() -> serde_json::Value {
    let mut ip = String::new();
    for _ in 0..4 {
        ip.push_str(&gen_range(0, 256).to_string());
        ip.push('.');
    }
    ip.pop();
    return serde_json::Value::String(ip);
}

pub fn gen_string(propname: &str, format: &Option<String>, is_quant: bool) -> serde_json::Value {
    // handle quantaties

    if is_quant {
        return rand_str_regex("([+-]?[0-9]+)(m|k|M|G|T|P|E)").into();
    }

    let lower = propname.to_lowercase();
    match lower {
        _ if lower == "name" => rand_str_regex("[a-z]{1,15}").into(),
        _ if lower.contains("port") => gen_range(0, 65535).to_string().into(),
        _ if lower.contains("ip") => gen_ip(),
        _ if lower == "host" => gen_ip(),
        _ if lower.contains("group") || lower == "runasuser" || lower.contains("username") => {
            gen_printable_string(
                gen_range(1, 15),
                Some("_-abcdefghijklmnopqrstuvwxyz".as_bytes()), // losely based on unix usernme naming rules
            )
            .into()
        }
        _ => {
            if format.is_some() {
                match format.as_ref().unwrap().as_str() {
                    "date-time" => rand_date_time().into(),
                    "int-or-string" => rand_int::<i64>().into(),
                    &_ => gen_printable_string(gen_range(1, 15), None).into(),
                }
            } else {
                gen_printable_string(gen_range(1, 15), None).into()
            }
        }
    }
}

pub fn gen_bool() -> serde_json::Value {
    serde_json::Value::Bool(gen_range(0, 2) == 1)
}

fn gen_int(propname: &str, format: &Option<String>) -> serde_json::Value {
    let lower = propname.to_lowercase();
    match lower {
        _ if lower.contains("port") => gen_range(0, 65535).into(),
        _ if lower.contains("group") || lower == "runasuser" || lower.contains("username") => {
            gen_range(0, 2147483647).into()
        }
        _ => {
            return match format {
                Some(fmt) => match fmt.as_str() {
                    "int32" => serde_json::Value::Number(rand_int::<i32>().into()),
                    "int64" => serde_json::Value::Number(rand_int::<i64>().into()),
                    &_ => serde_json::Value::Number(rand_int::<i64>().into()),
                },
                None => serde_json::Value::Number(rand_int::<i64>().into()),
            };
        }
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

    for _ in 0..gen_range(min, max + 1) {
        arr.as_array_mut()
            .unwrap()
            .push(gen_property(&items, propname));
    }
    return arr;
}

pub fn rand_enum_val(
    _enum: &Vec<serde_json::Value>,
    _enum_regex: &Vec<String>,
) -> Option<serde_json::Value> {
    // for json path there might be enums and regex enums defined
    // if both are defined, randomly choose one
    if !_enum.is_empty() && !_enum_regex.is_empty() {
        match gen_range(0, 2) {
            0 => {
                return Some(_enum[gen_range(0, _enum.len())].clone());
            }
            1 => {
                return Some(serde_json::Value::String(rand_str_regex(
                    &_enum_regex[gen_range(0, _enum_regex.len())],
                )));
            }

            _ => {}
        }
    }

    if !_enum.is_empty() {
        return Some(_enum[gen_range(0, _enum.len())].clone());
    }

    if !_enum_regex.is_empty() {
        return Some(rand_str_regex(&_enum_regex[gen_range(0, _enum_regex.len())]).into());
    }
    return None;
}

pub fn gen_property(spec: &K8sResourceSpec, propname: &str) -> serde_json::Value {
    // first check values and regex_values

    if let Some(val) = rand_enum_val(&spec._enum, &spec._enum_regex) {
        return val;
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
        "string" => return gen_string(propname, &spec.format, spec.is_quant),
        "boolean" => return gen_bool(),
        "array" => return gen_array(spec, propname),
        "integer" => return gen_int(propname, &spec.format),
        "number" => return gen_int(propname, &spec.format), // TODO: number differs from integer

        &_ => panic!("type not covered"),
    }
}

pub fn gen_resource(spec: &K8sResourceSpec) -> serde_json::Value {
    let mut resc = gen_property(spec, "$");

    // lastly we make sure that version, kind and name are correctly set
    // as they are required by the API

    resc["apiVersion"] = match &spec.group {
        Some(group) => format!("{}/{}", group, spec.version.as_ref().unwrap()),
        None => spec.version.as_ref().unwrap().clone(),
    }
    .into();

    resc["kind"] = spec.kind.clone().unwrap().into();

    resc["metadata"]["name"] = format!(
        "kubefuzz-{}",
        GENERATION_COUNT.fetch_add(1, Ordering::SeqCst)
    )
    .into();

    return resc;
}
