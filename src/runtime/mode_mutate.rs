use crate::args::Mutate;
use crate::error_exit;
use crate::generator::load_constrained_spec;
use crate::generator::K8sResourceSpec;
use crate::mutator::mutate_resource;
use serde_transcode::transcode;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn run(args: &Mutate) {
    // first load all constraint files

    let mut cmap = HashMap::<String, K8sResourceSpec>::new();

    for constraintfile in args.constraints[0].split(",") {
        let spec = load_constrained_spec(&constraintfile, &args.schemadir);

        let gvk = spec.gvk.as_ref().unwrap();

        if cmap.contains_key(gvk) {
            error_exit!(
                "Error loading constraints: duplicate gvk {}. Second key seen in {}",
                gvk,
                constraintfile
            );
        }

        cmap.insert(gvk.to_string(), spec);
    }

    let mut files_written: u64 = 0;

    // iterate over all resources
    // [0] because of clap missbehavior. see mode_generate.rs
    for resource in args.resources[0].split(",") {
        // read file as json
        let file = match File::open(resource) {
            Ok(file) => file,
            Err(e) => error_exit!("Error opening file {}: {}", resource, e),
        };

        // the manifests might be json or yaml.
        // internally, we prioritize json

        let ext = resource.split(".").last();

        let mut rawdata: serde_json::Value = match ext {
            Some("json") => {
                return serde_json::from_reader(&file).expect("Error parsing json in resource file")
            }

            Some("yaml") => {
                let mut buf = Vec::new();
                transcode(
                    serde_yaml::Deserializer::from_reader(&file),
                    &mut serde_json::Serializer::new(&mut buf),
                )
                .unwrap();
                serde_json::from_slice(&buf)
                    .expect("Internal error parsing transcode output from yaml to json")
            }

            _ => error_exit!(
                "Error parsing file {}: unknown file extension. Allowed are .json and .yaml",
                resource
            ),
        };

        // get gvk to identify right constraint
        let kind = match rawdata.get("kind") {
            Some(kind) => match kind.as_str() {
                Some(kind) => kind,
                None => error_exit!("Error parsing file {}: kind is not a string", resource),
            },
            None => error_exit!("Error parsing file {}: no kind found", resource),
        };

        let (group, version) = match rawdata.get("apiVersion") {
            Some(av) => match rawdata.get("apiVersion").unwrap().as_str() {
                Some(gv) => {
                    let gv: Vec<&str> = gv.split("/").collect();
                    match gv.len() {
                        1 => ("", gv[0]),
                        2 => (gv[0], gv[1]),
                        _ => error_exit!("Error parsing file {}: apiVersion malformed", resource),
                    }
                }
                None => error_exit!(
                    "Error parsing file {}: apiVersion is not a string",
                    resource
                ),
            },
            None => error_exit!("Error parsing file {}: no apiVersion found", resource),
        };

        let gvk = format!("{}.{}.{}", group, version, kind);

        let spec = match cmap.get(&gvk) {
            Some(spec) => spec,
            None => error_exit!(
                "Error parsing file {}: no constraint found for gvk {}",
                resource,
                gvk
            ),
        };

        // prepare directory

        let dir = PathBuf::from(&args.out).join(&gvk);

        match std::fs::create_dir_all(&dir) {
            Ok(_) => (),
            Err(e) => error_exit!("Error creating directory {}: {}", dir.display(), e),
        }

        for i in 0..args.num {
            mutate_resource(&mut rawdata, &spec);

            let mut file = match File::create(dir.with_file_name(match ext {
                Some("json") => format!("{}.json", i),
                Some("yaml") => format!("{}.yaml", i),
                _ => panic!("cant happen"), // as we checked the ext above
            })) {
                Ok(file) => file,
                Err(e) => error_exit!("Error opening file {}: {}", resource, e),
            };

            let content = match ext {
                Some("json") => serde_json::to_string_pretty(&rawdata).unwrap(),
                Some("yaml") => serde_yaml::to_string(&rawdata).unwrap(),
                _ => panic!("cant happen"), // as we checked the ext above
            };

            file.write_all(content.as_bytes())
                .expect("writing to mutated file failed");

            files_written += 1;
        }
    }

    info!("Done mutating. Written {} files", files_written);
}
