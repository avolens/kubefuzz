use crate::conf::ValuesMode;
use crate::generator::k8sresc::K8sResourceSpec;
use crate::{conf::ConstraintConfig, error_exit};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::{fs, path::PathBuf};

pub mod gen;
pub mod k8sresc;
pub mod rand;
/*
This struct represents the swagger spec json structure
of the k8s API. It has more fields like description and
multiple x- fields thtat we dont bother reading into
memory
*/

fn jsonpath_len(path: &str) -> usize {
    return path.split('.').filter(|x| !x.is_empty()).count();
}

fn normalize_path(path: &str) -> String {
    return path
        .split(".")
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>()
        .join(".");
}

fn iter_all_children(curpath: &str, curspec: &K8sResourceSpec, fullpaths: &mut Vec<String>) {
    // recursively iterates over all children, returning all terminal paths.
    // this method is used when setting required values based on a regex
    if curspec.properties.is_empty() && curspec.items.is_none() {
        fullpaths.push(curpath.to_string());
        return;
    }

    let children = if curspec._type == "array" {
        curspec.items.as_ref().unwrap().properties.keys()
    } else {
        curspec.properties.keys()
    };

    for child in children {
        let childpath = format!("{}.{}", curpath, child);
        let childspec = if curspec._type == "array" {
            curspec
                .items
                .as_ref()
                .unwrap()
                .properties
                .get(child)
                .unwrap()
        } else {
            curspec.properties.get(child).unwrap()
        };

        iter_all_children(&childpath, childspec, fullpaths);
    }
}

pub fn get_required_subs(
    ppath: &str,
    spec: &K8sResourceSpec,
    constraintconfig: &ConstraintConfig,
) -> HashSet<String> {
    let ppath_len = jsonpath_len(ppath);
    let mut required_subs: HashSet<String> = HashSet::new();

    for fcnfg in &constraintconfig.fields {
        // if we have a regex, we have to search all terminal
        // children paths for matches
        if fcnfg.regex {
            let mut terminal_children: Vec<String> = vec![];
            iter_all_children(ppath, spec, &mut terminal_children);

            let compiled = Regex::new(&fcnfg.path).expect("invalid regex");

            for child in terminal_children {
                if compiled.is_match(&child) {
                    // may be faster to check if we already inserted?
                    required_subs
                        .insert(child.split('.').collect::<Vec<&str>>()[ppath_len].to_string());
                }
            }
            continue;
        }

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

pub fn loadspec(specpath: &str) -> K8sResourceSpec {
    info!("Reading spec file: {:?}", specpath);
    let rawspec = fs::read_to_string(specpath).expect("Unable to read spec file");

    serde_json::from_str(&rawspec).expect("Unable to parse spec file")
}

fn constrain_spec(
    constraintconfig: &ConstraintConfig,
    spec: &mut K8sResourceSpec,
    parentpath: &String,
    paths_covered: &mut Vec<String>,
) {
    debug!(
        "constraining k8s resource spec. currently at {}",
        parentpath
    );

    /*
    This function slims down a given spec so it only contains
    the fields we care about during mutation. Further, it adds
    user supplied information like additional enums and overriden
    optional values
    */

    // add to this paths required values all children that are required
    // if this is a non terminal property
    if !spec.properties.is_empty() || spec.items.is_some() {
        for req_child in get_required_subs(&parentpath, &spec, &constraintconfig) {
            debug!(
                "adding '{}' to required values of {}",
                req_child, parentpath
            );

            if spec._type == "array" {
                spec.items.as_mut().unwrap().required.push(req_child);
            } else {
                spec.required.push(req_child);
            }
        }
    }

    // remove or recurse into subproperties depending on constraint
    let mut remove_properties = vec![];
    let mut recurse_properties = vec![];

    let required = match spec._type.as_str() {
        "array" => spec.items.as_ref().unwrap().required.clone(),
        _ => spec.required.clone(),
    };

    // if we have an array, we need to check the items property
    let toiter = if spec._type != "array" {
        spec.properties.iter_mut()
    } else {
        assert!(spec.properties.is_empty());
        spec.items.as_mut().unwrap().properties.iter_mut()
    };

    for (key, _subspec) in toiter {
        let subpath = format!("{}.{}", parentpath, key);

        if path_allowed(&subpath, &constraintconfig) || required.contains(key) {
            recurse_properties.push((key.clone(), subpath));
        } else {
            remove_properties.push(key.clone());
        }
    }

    // now check if the current path is an exact match
    // in which case we want to update some properties

    // iterate over all configs that target this path
    for fcnfg in constraintconfig
        .fields
        .iter()
        .filter(|fcnfg| match fcnfg.regex {
            true => {
                let re = Regex::new(&fcnfg.path).expect("invalid regex");
                re.is_match(&parentpath)
            }
            false => &fcnfg.path == parentpath,
        })
    {
        paths_covered.push(fcnfg.path.clone());

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
                            error_exit!(
                                "missing values_mode for field '{}' since enum is not empty : {:?}",
                                parentpath,
                                spec._enum
                            );
                        }
                    }
                }
            }
            None => {}
        }
        // maybe we also have regex values

        if fcnfg.regex_values.is_some() {
            spec._enum_regex = fcnfg.regex_values.as_ref().unwrap().clone();
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

    // if this a leaf node (terminal property), we can stop here
    if spec.properties.is_empty() && spec._type != "array" {
        return;
    }

    // check if we are removing required values
    for k in &remove_properties {
        if required.contains(k) {
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

fn verify_gvk(gvk: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = gvk.split(".").collect();
    if parts.len() != 3 {
        return Err("too few or too many parts in gvk. Expected 3, sperated by '.'".into());
    }

    if parts[1].is_empty() || parts[2].is_empty() {
        return Err("version or kind cannot be empty".into());
    }

    if !parts[2].chars().next().unwrap().is_uppercase() {
        return Err("kind must start with uppercase letter".into());
    }

    Ok(())
}

pub fn load_constrained_spec(constraintfile_path: &str, schemadir: &str) -> K8sResourceSpec {
    info!("Reading constraint file: {:?}", constraintfile_path);

    if !constraintfile_path.contains(".") {
        error_exit!(
            "invalid constraint file path '{}'. No extension",
            constraintfile_path
        );
    }

    let rawcontent =
        std::fs::read_to_string(constraintfile_path).expect("Unable to read constraint file");

    let mut constraint_config: ConstraintConfig =
        match constraintfile_path.split(".").last().unwrap() {
            "yaml" => serde_yaml::from_str(&rawcontent).expect("Unable to parse constraint file"),
            "json" => serde_json::from_str(&rawcontent).expect("Unable to parse constraint file"),

            &_ => {
                error_exit!(
                    "invalid constraint file path '{}'. Supported config languages: yaml, json",
                    constraintfile_path
                );
            }
        };

    // verify gvk
    match verify_gvk(&constraint_config.gvk) {
        Ok(_) => {}
        Err(e) => {
            error_exit!(
                "invalid gvk in constraint file '{}': {}",
                constraintfile_path,
                e
            );
        }
    }

    // verify regex
    for field in &constraint_config.fields {
        if field.regex_values.is_some() {
            for regex in field.regex_values.as_ref().unwrap() {
                match Regex::new(&regex) {
                    Ok(_) => {}
                    Err(e) => {
                        error_exit!(
                            "invalid regex in constraint file '{}': {}",
                            constraintfile_path,
                            e
                        );
                    }
                }
            }
        }
    }

    // normalize paths
    for fcnfg in &mut constraint_config.fields {
        if !fcnfg.regex {
            fcnfg.path = normalize_path(&fcnfg.path);
        }
    }

    let kind = constraint_config
        .gvk
        .split(".")
        .last()
        .unwrap()
        .to_lowercase();

    let fullpath = PathBuf::from(schemadir).join(&kind).with_extension("json");

    let mut spec = loadspec(fullpath.to_str().expect("invalid path"));

    spec.gvk = Some(constraint_config.gvk.clone());

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
                &kind,
                constraintfile_path
            );
        }
    }
    spec
}
