use std::{fs::canonicalize, process::Command};

use crate::{args::GetSchemas, error_exit};
pub fn run(args: &GetSchemas) {
    info!("getting schemas from {}", args.endpoint);

    // execute system command

    // ensure openapi2json command is available
    match Command::new("openapi2jsonschema")
        .args([
            "--stand-alone",
            format!(
                "-o {}",
                canonicalize(&args.out)
                    .expect("could not canonicalize path")
                    .to_str()
                    .unwrap()
            )
            .as_str(),
            &args.endpoint,
        ])
        .spawn()
    {
        Ok(_) => {}
        Err(e) => {
            error_exit!("Error while running openapi2json: {}.", e);
        }
    }
}
