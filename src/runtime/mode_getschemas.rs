use std::{fs::canonicalize, process::Command};

use crate::{args::GetSchemas, error_exit};
pub fn run(args: &GetSchemas) {
    info!("getting schemas from {}", args.endpoint);

    let mut child = match Command::new("openapi2jsonschema")
        .args(["--stand-alone", &args.endpoint])
        .current_dir(canonicalize(&args.out).expect("could not canonicalize path"))
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            error_exit!("Error while running openapi2json: {}.", e);
        }
    };

    child.wait().expect("could not wait for child process");
    info!("done.")
}
