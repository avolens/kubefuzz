use crate::conf::ValuesMode;
use crate::{conf::ConstraintConfig, error_exit};
use core::iter::Map;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::{collections::HashMap, fs::File};
use std::{fs, path::PathBuf};

pub mod gen;
pub mod rand;
/*
This struct represents the swagger spec json structure
of the k8s API. It has more fields like description and
multiple x- fields thtat we dont bother reading into
memory
*/

// TOOD: make it possible to supply $. as path
#[derive(Serialize, Deserialize, Debug)]
pub struct K8sResourceSpec {
    #[serde(rename = "type")]
    pub _type: String,

    #[serde(default)]
    pub properties: HashMap<String, Box<K8sResourceSpec>>,

    #[serde(rename = "enum", default)]
    pub _enum: Vec<serde_json::Value>,

    #[serde(default)]
    pub required: Vec<String>,

    pub minmax: Option<(usize, usize)>,

    pub items: Option<Box<K8sResourceSpec>>,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<K8sResourceSpec>>,
}

fn jsonpath_len(path: &str) -> usize {
    return path.split('.').filter(|x| !x.is_empty()).count();
}

pub fn get_required_subs(ppath: &str, constraintconfig: &ConstraintConfig) -> HashSet<String> {
    let ppath_len = jsonpath_len(ppath);
    let mut required_subs: HashSet<String> = HashSet::new();

    for fcnfg in &constraintconfig.fields {
        let fcnfg_parts = fcnfg.path.split('.').collect::<Vec<&str>>();
        let fcnfg_len = jsonpath_len(&fcnfg.path);

        debug!("checking if {} is a subpath of {}", fcnfg.path, ppath);
        if fcnfg_len <= ppath_len {
            continue;
        }

        if fcnfg.path.starts_with(&ppath) {
            debug!("{} is a subpath of {}", fcnfg.path, ppath);
            if fcnfg.required.is_some() {
                if fcnfg.required.unwrap() {
                    required_subs.insert(fcnfg_parts[ppath_len].to_string());
                }
            }
        }
    }
    return required_subs;
}

pub fn path_allowed(path: &str, constraintconfig: &ConstraintConfig) -> bool {
    let vectorized_path = path.split(".").collect::<Vec<&str>>();

    for each in &constraintconfig.fields {
        let vectorized_allow = each.path.split(".").collect::<Vec<&str>>();

        for (i, each) in vectorized_allow.iter().enumerate() {
            if each != &vectorized_path[i] {
                break;
            }
            if vectorized_allow.len() - 1 == i || vectorized_path.len() - 1 == i {
                return true;
            }
        }
    }

    return false;
}

pub fn loadspec(specname: &str) -> K8sResourceSpec {
    let fullpath = PathBuf::from("schemagen/schemas/")
        .join(specname.clone())
        .with_extension("json");

    info!("Reading spec file: {:?}", fullpath);
    let rawspec = fs::read_to_string(fullpath).expect("Unable to read spec file");

    serde_json::from_str(&rawspec).expect("Unable to parse spec file")
}

fn constrain_spec(
    constraintconfig: &ConstraintConfig,
    spec: &mut K8sResourceSpec,
    parentpath: &String,
    paths_covered: &mut Vec<String>,
) {
    debug!("filtering k8s resource spec. currently at {}", parentpath);

    /*
    This function slims down a given spec so it only contains
    the fields we care about during mutation. Further, it adds
    user supplied information like additional enums and overriden
    optional values
    */

    // add to this paths required values all children that are required
    for req_child in get_required_subs(&parentpath, &constraintconfig) {
        debug!(
            "adding '{}' to required values of {}",
            req_child, parentpath
        );
        if !spec.required.contains(&req_child) {
            if spec._type == "array" {
                spec.items.as_mut().unwrap().required.push(req_child);
            } else {
                spec.required.push(req_child);
            }
        }
    }
    // if this a leaf node (terminal property), we can stop here
    if spec.properties.is_empty() && spec._type != "array" {
        if !paths_covered.contains(parentpath) {
            paths_covered.push(parentpath.clone());
        }
        return;
    }

    // remove or recurse into subproperties depending on constraint
    let mut remove_properties = vec![];
    let mut recurse_properties = vec![];

    // if we have an array, we need to check the items property
    let toiter = if spec._type != "array" {
        spec.properties.iter_mut()
    } else {
        assert!(spec.properties.is_empty());
        spec.items.as_mut().unwrap().properties.iter_mut()
    };

    for (key, subspec) in toiter {
        let subpath = format!("{}.{}", parentpath, key);

        if path_allowed(&subpath, &constraintconfig) {
            recurse_properties.push((key.clone(), subpath));
        } else {
            remove_properties.push(key.clone());
        }
    }

    // now check if the current path is an exact match
    // in which case we want to update some properties
    match constraintconfig
        .fields
        .iter()
        .find(|fcnfg| &fcnfg.path == parentpath)
    {
        Some(fcnfg) => {
            paths_covered.push(parentpath.clone());

            // first, update the enum if needed
            match &fcnfg.values {
                Some(values) => {
                    if spec._enum.is_empty() {
                        spec._enum = values.clone();
                    } else {
                        match &fcnfg.values_mode {
                            Some(mode) => {
                                if *mode == ValuesMode::Override {
                                    warn!(
                                        "overriding enum for field '{}', original content : {:?}",
                                        parentpath, spec._enum
                                    );
                                    spec._enum = values.clone();
                                } else {
                                    spec._enum.extend(values.clone());
                                }
                            }
                            None => {
                                error_exit!("missing values_mode for field '{}' since enum is not empty : {:?}", parentpath,spec._enum);
                            }
                        }
                    }
                }
                None => {}
            }

            // also set min and max values for arrays
            match &fcnfg.minmax {
                Some(minmax) => {
                    if spec._type != "array" {
                        error_exit!(
                            "minmax is only allowed for arrays, but found for field '{}'",
                            parentpath
                        );
                    } else {
                        spec.minmax = Some(minmax.clone());
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    // remove all properties that are not on the allowlist
    // first get all required fields, depending on the type "array" or other
    let req_vals = match spec._type.as_str() {
        "array" => &spec.items.as_ref().unwrap().required,
        &_ => &spec.required,
    };

    // check if we are removing required values
    for k in &remove_properties {
        if req_vals.contains(k) {
            warn!(
                "removing a required field '{}.{}'. K8s will API probably reject this",
                parentpath, &k
            );
        }
    }

    // now remove
    for k in &remove_properties {
        debug!("removing property {}", format!("{}.{}", parentpath, &k));
        // if we are dealing with an array, we have to delete from items.properties
        if spec._type == "array" {
            spec.items.as_mut().unwrap().properties.remove(k).unwrap();
            spec.items.as_mut().unwrap().required.retain(|x| x != k);
        } else {
            spec.properties.remove(k).unwrap();
            spec.required.retain(|x| x != k);
        }
    }

    // apply allowlist to all sub properties
    for (prop, path) in recurse_properties {
        // if we are dealing with an array, we have to recurse into items.properties
        let resc: &mut Box<K8sResourceSpec> = if spec._type == "array" {
            spec.items
                .as_mut()
                .unwrap()
                .properties
                .get_mut(&prop)
                .expect("missing prop")
        } else {
            spec.properties.get_mut(&prop).expect("missing prop")
        };
        constrain_spec(&constraintconfig, &mut *resc, &path, paths_covered);
    }
}

pub fn load_constrained_spec(constraintfile_path: &str, specname: &str) -> K8sResourceSpec {
    info!("Reading constraint file: {:?}", constraintfile_path);

    if !constraintfile_path.contains(".") {
        error_exit!(
            "invalid constraint file path '{}'. No extension",
            constraintfile_path
        );
    }

    let rawcontent =
        std::fs::read_to_string(constraintfile_path).expect("Unable to read constraint file");

    let constraint_config: ConstraintConfig = match constraintfile_path.split(".").last().unwrap() {
        "yaml" => serde_yaml::from_str(&rawcontent).expect("Unable to parse constraint file"),
        "json" => serde_json::from_str(&rawcontent).expect("Unable to parse constraint file"),

        &_ => {
            error_exit!(
                "invalid constraint file path '{}'. Supported config languages: yaml, json",
                constraintfile_path
            );
        }
    };

    let mut spec = loadspec(specname);

    // first collect all allowed jsonpath into simple list

    let mut paths_covered: Vec<String> = vec![];
    constrain_spec(
        &constraint_config,
        &mut spec,
        &String::from("$"),
        &mut paths_covered,
    );

    for paths in constraint_config.fields {
        if !paths_covered.contains(&paths.path) {
            error_exit!(
                "invalid path '{}' for spec '{}' stemming from constraintfile '{}'",
                paths.path,
                specname,
                constraintfile_path
            );
        }
    }
    spec
}
