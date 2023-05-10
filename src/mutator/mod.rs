use crate::conf::ConstraintConfig;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct K8sResourceSpec {
    pub resource_name: String,
    pub resource_spec: serde_json::Value,
}

pub fn loadspec(specname: &str) -> K8sResourceSpec {
    let fullpath = PathBuf::from("schemagen/schemas/")
        .join(specname)
        .with_extension("json");

    info!("Reading spec file: {:?}", fullpath);
    let rawspec = fs::read_to_string(fullpath).expect("Unable to read spec file");

    serde_json::from_str(&rawspec).expect("Unable to parse spec file")
}

pub fn apply_constaintfile(path: &str, spec: &K8sResourceSpec) -> K8sResourceSpec {
    let rawcontent = std::fs::read_to_string(path).expect("Unable to read constraint file");

    let constraint_config: ConstraintConfig =
        serde_json::from_str(&rawcontent).expect("Unable to parse constraint file");

    match constraint_config.mode {
        crate::conf::Mode::Whitelist => {
            // start with the original spec
            // and gradually remove fields
            let mut new_spec = spec.clone();
        }

        crate::conf::Mode::Blacklist => {
            let mut new_spec = serde_json::Value::default();
        }
    }
}
