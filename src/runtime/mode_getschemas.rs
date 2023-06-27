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

    match child.wait() {
        Ok(status) => {
            if !status.success() {
                error_exit!("openapi2jsonschema failed with status: {}", status);
            }
        }
        Err(e) => {
            error_exit!("Error while waiting for openapi2jsonschema: {}", e);
        }
    }
    info!("done.")
}
