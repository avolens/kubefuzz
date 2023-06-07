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
        //load_constrained_spec(constrait);
        let cspec = load_constrained_spec(file, &args.schemadir);

        // in case gvk starts with an empty group
        let gvk = cspec.gvk.as_ref().unwrap();
        let gvk_dir = match gvk.strip_prefix(".") {
            Some(s) => s,
            None => gvk,
        };

        // make directory
        // .unwrap is safe here because of load_constrained_spec
        let dir = PathBuf::from(&args.out).join(&gvk_dir);

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
            let res = match serde_yaml::to_string(&gen_resource(&cspec)) {
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
        }
    }

    info!("Done generating. Written {} files", files_written);
}
