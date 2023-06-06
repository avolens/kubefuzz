use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::Path;

fn is_dir(value: &str) -> Result<(String), String> {
    let path = Path::new(&value);
    if path.is_dir() {
        Ok((path.to_str().expect("could not read path").to_string()))
    } else {
        Err(String::from(
            "Specified path is not a directory or does not exist",
        ))
    }
}

pub fn is_file(value: &str) -> Result<(), String> {
    let path = Path::new(&value);
    if path.is_file() {
        Ok(())
    } else {
        Err(String::from(
            "Specified path is not a file or does not exist",
        ))
    }
}

fn is_all_files(arr: &str) -> Result<String, String> {
    let paths = arr.split(",");
    for path in paths {
        match is_file(path) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "Specified path '{}' is not a file or does not exist",
                    path
                ))
            }
        }
    }
    Ok(arr.to_string())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub action: Action,

    /// seed to use
    #[arg(short, long)]
    pub seed: Option<u64>,
}

#[derive(Parser, Debug)]
pub enum Action {
    #[clap(name = "generate")]
    Generate(Generate),
    #[clap(name = "mutate")]
    Mutate(Mutate),
    #[clap(name = "fuzz")]
    Fuzz(Fuzz),
    #[clap(name = "getresources")]
    GetResources(GetResources),
}

#[derive(Parser, Debug)]
pub struct Generate {
    /// comma seperated list of constraint files to apply
    #[arg(short, long, value_parser = is_all_files, required = true)]
    pub constraints: Vec<String>,

    /// directory containing k8s json resource specs
    #[arg(short, long,value_parser = is_dir)]
    pub specdir: String,

    #[arg(short, long)]
    /// number of resources to generate
    pub num: Option<u32>,
}

#[derive(Parser, Debug)]
pub struct Fuzz {}
#[derive(Parser, Debug)]
pub struct Mutate {}
#[derive(Parser, Debug)]
pub struct GetResources {}
