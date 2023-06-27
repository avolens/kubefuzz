use crate::args::Generate;
use crate::error_exit;
use crate::generator::gen::gen_resource;
use crate::generator::load_constrained_spec;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn run(args: &Generate) {
    // iterate over all constraints -> load constrained specs -> generate

    // we need to manually split the constraint files again, since
    // clap does not allow to do custom value parsing and return Vec<String>
    // so the whole argument is actually contained in args.constraints[0]

    let constraintfiles = args.constraints[0].split(",");

    let mut files_written: u64 = 0;

    for file in constraintfiles {
        let cspec = load_constrained_spec(file, &args.schemadir);

        // make directory
        // .unwrap is safe here because of load_constrained_spec
        let mut gvkdir = cspec.gvk.clone().unwrap().replace("/", "");

        match gvkdir.strip_prefix(".") {
            Some(s) => gvkdir = s.to_string(),
            None => {}
        }

        let dir = PathBuf::from(&args.out).join(&gvkdir);

        println!(
            "Generating {} resources in {}",
            args.num,
            dir.to_str().unwrap()
        );
        match std::fs::create_dir_all(&dir) {
            Ok(_) => {}
            Err(e) => {
                error_exit!(
                    "could not create directory {}: {}",
                    dir.to_str().unwrap(),
                    e
                );
            }
        }

        // write files
        for i in 0..args.num {
            let res = match serde_json::to_string_pretty(&gen_resource(&cspec)) {
                Ok(res) => res,
                Err(e) => {
                    error!("could not serialize resource: {}", e);
                    continue;
                }
            };

            let file = dir.join(format!("{}.yaml", i));

            let mut f = match File::create(&file) {
                Ok(f) => f,
                Err(e) => {
                    error_exit!("could not create file {}: {}", file.to_str().unwrap(), e);
                }
            };

            f.write_all(res.as_bytes()).unwrap();
            files_written += 1;
        }
    }

    info!("Done generating. Written {} files", files_written);
}
