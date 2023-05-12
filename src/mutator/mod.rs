use crate::{conf::ConstraintConfig, error_exit};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct K8sResourceSpec {
    pub resource_name: String,
    pub resource_spec: serde_json::Value,
}

pub fn loadspec(specname: String) -> K8sResourceSpec {
    let fullpath = PathBuf::from("schemagen/schemas/")
        .join(specname.clone())
        .with_extension("json");

    info!("Reading spec file: {:?}", fullpath);
    let rawspec = fs::read_to_string(fullpath).expect("Unable to read spec file");

    K8sResourceSpec {
        resource_name: specname,
        resource_spec: serde_json::from_str(&rawspec).expect("Unable to parse spec file"),
    }
}

fn apply_whitelist_to_schema(
    allowlist: &mut Vec<String>,
    mut object: serde_json::Value,
    jsonpath: &String,
) -> serde_json::Value {
    debug!("filtering k8s resource spec. currently at {}", jsonpath);

    // if this a leaf node (terminal property), we can stop here
    if !object.as_object().unwrap().contains_key("properties") {
        return object;
    }

    // TODO: keep track of unused/invalid json paths!

    let required_fields: Vec<String> = match object.as_object().unwrap().get("required") {
        Some(v) => v
            .as_array()
            .expect("expected array in 'required' field")
            .iter()
            .map(|v| {
                v.as_str()
                    .expect("expected string elements only in 'required' field")
                    .to_string()
                    .clone()
            })
            .collect(),
        None => vec![],
    };

    let properties = object
        .as_object_mut()
        .unwrap()
        .get_mut("properties")
        .expect("'properties' key not found in schema");

    // now look at all possible properties

    let mut remove_properties = vec![];
    let mut recurse_properties = vec![];

    for (key, mut value) in properties.as_object().unwrap() {
        let new_path = format!("{}.{}", jsonpath, key);

        // if we whitelisted the whole key, we can skip it.
        // we also dont have to go into it, thus it goes neither
        // to remove or recurse

        if allowlist.contains(&new_path) {
            // update whitelist so caller sees every jsonpath not used.
            // we can also remove any path that starts with this one
            // we can now delete all paths that start with this one
            let mut positions: Vec<usize> = allowlist
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if s.starts_with(&new_path) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            // so we dont deal with shifting indices
            positions.reverse();
            for p in positions {
                debug!("retaining property  {}", allowlist[p]);
                allowlist.remove(p);
            }

            // do not recurse or remove the path. We can simply
            // ignore this whole subtree
            continue;
        }

        if !(allowlist.iter().any(|s| s.starts_with(&new_path))) {
            // value does not match any whitelist path partially
            remove_properties.push(key.clone());
        } else {
            // value does match partially. We have to go deeper
            recurse_properties.push((key.clone(), new_path));
        }
    }

    // remove all properties that are not on the whitelist
    for k in remove_properties {
        debug!("removing property {}", k);

        if required_fields.contains(&k) {
            warn!(
                "removing a required field '{}.{}'. K8s will API probably reject this",
                jsonpath, &k
            );
        }

        properties.as_object_mut().unwrap().remove(&k);
    }

    // apply whitelist to all sub properties
    for (k, p) in recurse_properties {
        let changed_value = properties.as_object_mut().unwrap().get_mut(&k).unwrap();
        *changed_value = apply_whitelist_to_schema(allowlist, changed_value.clone(), &p);
    }

    object
}

pub fn load_constrained_spec(
    constraintfile_path: &str,
    mut spec: K8sResourceSpec,
) -> K8sResourceSpec {
    info!("Reading constraint file: {:?}", constraintfile_path);
    let rawcontent =
        std::fs::read_to_string(constraintfile_path).expect("Unable to read constraint file");

    let constraint_config: ConstraintConfig =
        serde_json::from_str(&rawcontent).expect("Unable to parse constraint file");

    // first collect all allowed jsonpath into simple list

    let mut allowlist: Vec<String> = constraint_config
        .fields
        .iter()
        .map(|field| match field {
            crate::conf::FieldConfig::String(s) => s.clone(),
            crate::conf::FieldConfig::Struct(s) => s.path.clone(),
        })
        .collect();

    spec.resource_spec =
        apply_whitelist_to_schema(&mut allowlist, spec.resource_spec, &String::from("$"));

    for w in allowlist {
        error_exit!(
            "invalid path '{}' for spec '{}' stemming from constraintfile '{}'",
            w,
            spec.resource_name,
            constraintfile_path
        );
    }
    spec
}
