use crate::conf::ValuesMode;
use crate::generator::k8sresc::K8sResourceSpec;
use crate::{conf::ConstraintConfig, error_exit};
use regex::Regex;
use serde_yaml;
use std::collections::HashMap;
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

#[derive(PartialEq, Debug)]
enum ChildAction {
    Allowed,
    Required,
    Forbidden,
}

fn get_children_status(
    ppath: &str,
    spec: &K8sResourceSpec,
    constraintconfig: &ConstraintConfig,
    paths_covered: &mut Vec<String>,
) -> HashMap<String, ChildAction> {
    // this function maps each child of the parent path to a ChildAction

    // at the start, all children are mapped to forbidden
    let mut children: HashMap<String, ChildAction> = match spec._type.as_str() {
        "array" => spec.items.as_ref().unwrap().properties.keys(),
        "object" => spec.properties.keys(),
        _ => panic!("unexpected type in spec"),
    }
    .map(|x| (format!("{}.{}", ppath, x), ChildAction::Forbidden))
    .collect();

    // first go through all non remove fieldconfigs
    for fcfg in constraintconfig.fields.iter().filter(|x| !x.remove) {
        // go through all children that are still forbidden
        for (childpath, action) in children
            .iter_mut()
            // we care for fields still forbidden or allowed (latter could become required)
            .filter(|(_, x)| **x != ChildAction::Required)
        {
            // is_allowed is computed differently based on if we are dealing with
            // a regex or a normal path
            let is_allowed = match fcfg.regex {
                false => path_allowed(childpath, &fcfg.path),
                true => {
                    // regex path. In addition we need to go through all
                    // possible subpaths in order to check for egex matches

                    let compiled = Regex::new(&fcfg.path).expect("invalid regex");
                    let mut subpaths: Vec<String> = vec![];
                    iter_all_children(ppath, spec, &mut subpaths);

                    let mut found = false;
                    for subpath in subpaths {
                        if compiled.is_match(&subpath) && path_allowed(childpath, &subpath) {
                            found = true;
                            break;
                        }
                    }
                    found
                }
            };

            // if the child is allowed, we set it to allowed or required
            if is_allowed {
                *action = match fcfg.required {
                    true => ChildAction::Required,
                    false => ChildAction::Allowed,
                };
            }
        }
    }
    // lastly, we might put some paths from allowed/required
    // back into forbidden, because "remove" has a higher priority
    for fcfg in constraintconfig.fields.iter().filter(|x| x.remove) {
        // we just need to look at the childpaths
        // that are now allowed or required

        for (childpath, action) in children
            .iter_mut()
            .filter(|(_, x)| **x != ChildAction::Forbidden)
        {
            // again, the computation depends on if we are dealing with
            // regex paths

            let should_remove = match fcfg.regex {
                // here, we only want to remove if the childpath
                // is exactly the one in the fieldconfig
                false => &fcfg.path == childpath,
                true => Regex::new(&fcfg.path)
                    .expect("invalid regex")
                    .is_match(&childpath),
            };
            if should_remove {
                *action = ChildAction::Forbidden;
                // since we wont "touch" this path, we have to
                // add the config path to the covered ones right here
                paths_covered.push(fcfg.path.clone());
            }
        }
    }

    children
}

pub fn path_allowed(path: &str, allow: &str) -> bool {
    let vectorized_path = path.split(".").collect::<Vec<&str>>();
    let vectorized_allow = allow.split(".").collect::<Vec<&str>>();

    for (i, each) in vectorized_allow.iter().enumerate() {
        if each != &vectorized_path[i] {
            break;
        }
        if vectorized_allow.len() - 1 == i || vectorized_path.len() - 1 == i {
            return true;
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

    // remove or recurse into subproperties depending on constraint
    let mut remove_properties = vec![];
    let mut recurse_properties = vec![];

    // if this is a non terminal property, we will check its children
    // to determine which we can remove, which to recurse into and which
    // to additionally set to "required"
    if !spec.properties.is_empty() || spec.items.is_some() {
        for (childpath, action) in
            get_children_status(parentpath, spec, constraintconfig, paths_covered)
        {
            let child = childpath.split('.').last().unwrap().to_string();
            match action {
                ChildAction::Forbidden => remove_properties.push(child),
                ChildAction::Allowed => recurse_properties.push((child, childpath)),
                ChildAction::Required => {
                    recurse_properties.push((child.clone(), childpath));

                    // where we save required to depends on type

                    match spec._type.as_str() {
                        "array" => spec.items.as_mut().unwrap().required.push(child),
                        _ => spec.required.push(child),
                    }
                }
            }
        }
    }

    let required = match spec._type.as_str() {
        "array" => spec.items.as_ref().unwrap().required.clone(),
        _ => spec.required.clone(),
    };

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
                "removing a required field '{}.{}'. K8s API could reject this",
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
    if constraint_config.version.is_empty() {
        error_exit!("{}: version cannot be empty", constraintfile_path);
    }
    if constraint_config.kind.is_empty() {
        error_exit!("{}: kind cannot be empty", constraintfile_path);
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

    let kind = constraint_config.kind.to_lowercase();

    let fullpath = PathBuf::from(schemadir).join(&kind).with_extension("json");

    let mut spec = loadspec(fullpath.to_str().expect("invalid path"));

    spec.gvk = Some(format!(
        "{}.{}.{}",
        constraint_config.group, constraint_config.version, constraint_config.kind
    ));

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
