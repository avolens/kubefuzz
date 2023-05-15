use crate::{conf::ConstraintConfig, error_exit};
use core::iter::Map;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
use std::{fs, path::PathBuf};
// TODO:
// 2. add fields from constraint file => This will clash with whitelist removing!!! find out a way to do this efficientlyj
/*
This struct represents the swagger spec json structure
of the k8s API. It has more fields like description and
multiple x- fields thtat we dont bother reading into
memory
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct K8sResourceSpec {
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(default)]
    pub properties: HashMap<String, Box<K8sResourceSpec>>,

    #[serde(rename = "enum", default)]
    pub _enum: Vec<String>,

    #[serde(default)]
    pub required: Vec<String>,

    pub items: Option<Box<K8sResourceSpec>>,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<K8sResourceSpec>>,
}

pub fn loadspec(specname: &str) -> K8sResourceSpec {
    let fullpath = PathBuf::from("schemagen/schemas/")
        .join(specname.clone())
        .with_extension("json");

    info!("Reading spec file: {:?}", fullpath);
    let rawspec = fs::read_to_string(fullpath).expect("Unable to read spec file");

    serde_json::from_str(&rawspec).expect("Unable to parse spec file")
}

fn constrain_spec(allowlist: &mut Vec<String>, spec: &mut K8sResourceSpec, jsonpath: &String) {
    debug!("filtering k8s resource spec. currently at {}", jsonpath);

    /*
    This function slims down a given spec so it only contains
    the fields we care about during mutation. Further, it adds
    user supplied information like addiotnal enums and overriden
    optional values
    */

    // if this a leaf node (terminal property), we can stop here
    if spec.properties.is_empty() {
        return;
    }

    // now look at all possible properties

    let mut remove_properties = vec![];
    let mut recurse_properties = vec![];

    for (key, mut value) in spec.properties.iter() {
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

        if spec.required.contains(&k) {
            warn!(
                "removing a required field '{}.{}'. K8s will API probably reject this",
                jsonpath, &k
            );
        }

        spec.properties.remove(&k).unwrap();
    }

    // apply allowlist to all sub properties
    for (prop, path) in recurse_properties {
        let prop: &mut Box<K8sResourceSpec> = spec.properties.get_mut(&prop).expect("missing prop");
        constrain_spec(allowlist, &mut *prop, &path);
    }
}

pub fn load_constrained_spec(constraintfile_path: &str, specname: &str) -> K8sResourceSpec {
    info!("Reading constraint file: {:?}", constraintfile_path);
    let rawcontent =
        std::fs::read_to_string(constraintfile_path).expect("Unable to read constraint file");

    let constraint_config: ConstraintConfig =
        serde_json::from_str(&rawcontent).expect("Unable to parse constraint file");

    let mut spec = loadspec(specname);

    // first collect all allowed jsonpath into simple list

    let mut allowlist: Vec<String> = constraint_config
        .fields
        .iter()
        .map(|field| match field {
            crate::conf::FieldConfig::String(s) => s.clone(),
            crate::conf::FieldConfig::Struct(s) => s.path.clone(),
        })
        .collect();

    constrain_spec(&mut allowlist, &mut spec, &String::from("$"));

    for w in allowlist {
        error_exit!(
            "invalid path '{}' for spec '{}' stemming from constraintfile '{}'",
            w,
            specname,
            constraintfile_path
        );
    }
    spec
}
