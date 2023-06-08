use crate::generator::{
    gen::{gen_bool, rand_enum_val},
    rand::{rand_int, rand_jkh},
    K8sResourceSpec,
};
use serde_json::Value;
use std::collections::HashMap;

fn mutate_bool(resource: &mut serde_json::Value, constraint: &K8sResourceSpec) {
    // simple, we just always generate a new value
    *resource = gen_bool();
}

fn mutate_number(resource: &mut serde_json::Value, constraint: &K8sResourceSpec) {
    // thre ecases: magic values, arithmetic , random conform value
    // todo: type confusion

    match gen_range(0, 3) {
        0 => {
            // magic values
        }
        1 => {
            // arithmetic
        }
        2 => {
            // random conform value
            *resource = match constraint.format 
        }
        _ => {
            panic!("wrong range");
        }
    }
}

fn mutate_object(resource: &mut serde_json::Value, constraint: &K8sResourceSpec) {
    // first value enum
    if let Some(val) = rand_enum_val(&constraint._enum, &constraint._enum_regex) {
        *resource = val;
        return;
    }

    // todo : type confusion

    match resource {
        Value::Bool => mutate_bool(resource, constraint),
        Value::Number => mutate_number(resource, constraint),
        Value::String => mutate_string(resource, constraint),
        Value::Array => mutate_array(resource, constraint),
        Value::Object => mutate_resource(resource, constraint),
    }
}

pub fn mutate_resource(resource: &mut serde_json::Value, constraintcfg: &K8sResourceSpec) {
    // mutate a resource based on its constrained spec. We mutate valuees according to
    // the enums set, on its type  and on its format.

    // start by calling mutate object on the root of the resource

    mutate_object(resource, constraintcfg);
}
