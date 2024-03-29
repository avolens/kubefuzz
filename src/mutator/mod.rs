use crate::generator::{
    gen::{gen_bool, gen_property, gen_string, rand_enum_val},
    k8sresc::K8sResourceSpec,
    rand::{chance, gen_printable_string, gen_range, rand_int, rand_str_regex, shuffle},
};
use serde_json::Value;

fn mutate_bool(resource: &mut serde_json::Value) {
    // simple, we just always generate a new value
    *resource = gen_bool();
}

fn mutate_number(resource: &mut serde_json::Value, constraint: &K8sResourceSpec) {
    // three cases: magic values, arithmetic , random conform value

    match gen_range(0, 3) {
        0 => {
            // magic values
            *resource = match constraint.format.as_deref() {
                Some("int32") => match gen_range(0, 4) {
                    0 => Value::Number(0.into()),
                    1 => Value::Number(i32::MIN.into()),
                    2 => Value::Number(i32::MAX.into()),
                    3 => Value::Number((-1).into()),
                    _ => panic!(""),
                },
                Some("int64") => match gen_range(0, 4) {
                    0 => Value::Number(0.into()),
                    1 => Value::Number(i64::MIN.into()),
                    2 => Value::Number(i64::MAX.into()),
                    3 => Value::Number((-1).into()),
                    _ => panic!(""),
                },
                Some(_) => {
                    panic!("number format not covered")
                }
                None => match gen_range(0, 6) {
                    0 => Value::Number(0.into()),
                    1 => Value::Number(i32::MAX.into()),
                    2 => Value::Number(i32::MIN.into()),
                    3 => Value::Number(i64::MAX.into()),
                    4 => Value::Number(i64::MIN.into()),
                    5 => Value::Number((-1).into()),
                    _ => panic!(""),
                },
            }
        }
        1 => {
            // arithmetic
            *resource = match constraint.format.as_deref() {
                Some("int32") => (resource.as_i64().unwrap() as i32)
                    .wrapping_add(gen_range(-1, 2))
                    .into(),
                None | Some("int64") => resource
                    .as_i64()
                    .unwrap()
                    .wrapping_add(gen_range(-1, 2))
                    .into(),
                Some(_) => {
                    panic!("number format not covered")
                }
            }
        }
        2 => {
            // random conform value
            *resource = match constraint.format.as_deref() {
                Some("int32") => Value::Number(rand_int::<i32>().into()),
                Some("int64") => Value::Number(rand_int::<i64>().into()),
                None => Value::Number(rand_int::<i64>().into()),
                Some(_) => {
                    panic!("number format not covered")
                }
            }
        }
        _ => panic!(""),
    }
}
fn mutate_string(resource: &mut serde_json::Value, constraint: &K8sResourceSpec, curpath: &str) {
    // empty string, random string, random conform string
    debug!("mutating string {}", curpath);

    if constraint.is_quant {
        *resource = rand_str_regex("([+-]?[0-9]+)(m|k|M|G|T|P|E)").into()
    }

    *resource = match gen_range(0, 3) {
        0 => "".into(),
        1 => gen_printable_string(gen_range(1, 15), None).into(),
        2 => gen_string(curpath, &constraint.format, constraint.is_quant),
        _ => panic!(""),
    }
}

fn mutate_array(resource: &mut serde_json::Value, constraint: &K8sResourceSpec, curpath: &str) {
    // 1. we might remove some elements

    debug!("mutating array at {}", curpath);

    shuffle(resource.as_array_mut().unwrap());

    for _ in 0..gen_range(0, resource.as_array().unwrap().len()) {
        resource.as_array_mut().unwrap().pop();
    }

    // 2. we might mutate all elements

    // constraint already is .items
    for obj in resource.as_array_mut().unwrap() {
        match obj {
            Value::Object(_) => {
                mutate_object(obj, constraint, curpath);
            }
            Value::Array(_) => {
                mutate_array(obj, constraint, curpath);
            }
            Value::String(_) => {
                mutate_string(obj, constraint, curpath);
            }
            Value::Number(_) => {
                mutate_number(obj, constraint);
            }
            Value::Bool(_) => {
                mutate_bool(obj);
            }
            Value::Null => {
                panic!("null value in array")
            }
        }
    }

    // 3. we might add new elements

    for _ in 0..gen_range(0, 5) {
        resource
            .as_array_mut()
            .unwrap()
            .push(gen_property(constraint, curpath))
    }
}

fn mutate_object(resource: &mut serde_json::Value, constraint: &K8sResourceSpec, curpath: &str) {
    // first value enum
    debug!("mutating object {}", curpath);
    if let Some(val) = rand_enum_val(&constraint._enum, &constraint._enum_regex) {
        *resource = val;
        return;
    }

    // 1. we might remove some non required fields

    let required = match constraint._type.as_str() {
        "array" => &constraint.items.as_ref().unwrap().required,
        _ => &constraint.required,
    };

    let toremove: Vec<String> = resource
        .as_object()
        .unwrap()
        .keys()
        .filter(|fieldname| !required.contains(fieldname) && chance(0.1))
        .map(|s| s.clone())
        .collect();

    for fieldname in toremove {
        let subpath = format!("{}.{}", curpath, fieldname);
        if subpath == "$.apiVersion"
            || subpath == "$.kind"
            || subpath == "$.metadata"
            || subpath == "$.metadata.name"
        {
            continue;
        }
        debug!("removing field {}", subpath);
        resource.as_object_mut().unwrap().remove(&fieldname);
    }

    // 2. we might mutate all fieds

    for (key, field) in resource.as_object_mut().unwrap() {
        if chance(0.1) {
            continue;
        }
        let subpath = format!("{}.{}", curpath, key);
        if subpath == "$.apiVersion"
            || subpath == "$.kind"
            || subpath == "$.metadata"
            || subpath == "$.metadata.name"
        {
            continue;
        }
        debug!("mutating field {}", subpath);

        let subconstraint = match constraint._type.as_str() {
            "array" => constraint.items.as_ref().unwrap().properties.get(key),
            _ => constraint.properties.get(key),
        }
        .unwrap();

        match field {
            Value::Bool(_) => mutate_bool(field),
            Value::Number(_) => mutate_number(field, subconstraint),
            Value::String(_) => mutate_string(field, subconstraint, &key),
            Value::Array(_) => mutate_array(field, subconstraint, &subpath),
            Value::Object(_) => mutate_object(field, subconstraint, &subpath),
            Value::Null => {}
        }
    }

    // 3. we might add some fields

    let toadd: Vec<String> = constraint
        .properties
        .keys()
        .filter(|fieldname| !resource.as_object().unwrap().contains_key(*fieldname))
        .map(|s| s.clone())
        .collect();

    for fieldname in toadd {
        let subpath = format!("{}{}.", curpath, fieldname);
        debug!("adding field {}", subpath);
        resource.as_object_mut().unwrap().insert(
            fieldname.clone(),
            gen_property(&constraint.properties[&fieldname], &subpath),
        );
    }
}

pub fn mutate_resource(resource: &mut serde_json::Value, constraintcfg: &K8sResourceSpec) {
    // mutate a resource based on its constrained spec. We mutate valuees according to
    // the enums set, on its type  and on its format.

    // start by calling mutate object on the root of the resource
    mutate_object(resource, constraintcfg, "$");
}
