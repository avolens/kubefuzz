use clap::Parser;
use std::path::Path;

fn is_dir(value: &str) -> Result<String, String> {
    let path = Path::new(&value);
    if path.is_dir() {
        Ok(path.to_str().expect("could not read path").to_string())
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
    // seems to be a bug in clap, we cant return a Vec<String> here
    let paths = arr.split(",");
    for path in paths {
        match is_file(path) {
            Ok(_) => {}
            Err(_e) => {
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
    /// generate manifests with constraints
    #[clap(name = "generate")]
    Generate(Generate),

    /// mutate existing manifests with constraints
    #[clap(name = "mutate")]
    Mutate(Mutate),

    /// fuzz admission controller chain with constraints
    #[clap(name = "fuzz")]
    Fuzz(Fuzz),

    /// get json schemas from k8s api
    #[clap(name = "get-schemas")]
    GetSchemas(GetSchemas),
}

#[derive(Parser, Debug)]
pub struct Generate {
    /// comma seperated list of constraint files to apply
    #[arg(short, long, value_parser = is_all_files, required = true)]
    pub constraints: Vec<String>,

    /// directory containing k8s json resource schemas
    #[arg(short, long,value_parser = is_dir)]
    pub schemadir: String,

    /// output direcotry of generated schemas
    #[arg(short, long,value_parser = is_dir)]
    pub out: String,

    /// number of manifests to generate per resource
    #[arg(short, long, default_value = "10")]
    pub num: u32,
}

#[derive(Parser, Debug)]
pub struct Fuzz {
    /// optional custom path to kubeconfig
    #[arg(short, long)]
    pub kubeconfig: Option<String>,

    /// comma seperated list of constraint files to apply
    #[arg(short, long, value_parser = is_all_files, required = true)]
    pub constraints: Vec<String>,

    /// directory containing k8s json resource schemas
    #[arg(short, long,value_parser = is_dir,required = true)]
    pub schemadir: String,

    /// namespace to use while fuzzing
    #[arg(short, long, default_value = "default")]
    pub namespace: String,

    /// directory to save and update fuzzing results
    #[arg(short, long,value_parser = is_dir)]
    pub fuzzdir: String,

    /// time in seconds until an api request is considered timed out
    #[arg(short, long, default_value = "5")]
    pub timeout: u32,

    /// maximum number of entries in the corpus in memory
    #[arg(short, long, default_value = "50")]
    pub max_corpus_count: usize,
}

#[derive(Parser, Debug)]
pub struct Mutate {
    /// comma seperated list of resources to be mutated
    #[arg(short, long, required = true,value_parser = is_all_files)]
    pub resources: Vec<String>,

    /// directory containing k8s json resource schemas
    #[arg(short, long,value_parser = is_dir)]
    pub schemadir: String,

    /// output directory of mutated resources
    #[arg(short, long,value_parser = is_dir)]
    pub out: String,

    /// number of mutated resources to generate per resource
    #[arg(short, long, default_value = "10")]
    pub num: u32,

    /// comma seperated list of constraint files to apply
    #[arg(short, long, value_parser = is_all_files, required = true)]
    pub constraints: Vec<String>,

    /// max number of samples saved into fuzzing directory
    #[arg(short, long, default_value = "50")]
    pub max_samples: usize,
}

#[derive(Parser, Debug)]
pub struct GetSchemas {
    /// openapi endpoint of k8s api, typically at /openapi/v2
    #[arg(short, long)]
    pub endpoint: String,

    /// directory in which the 'schemas' directory will be created
    #[arg(short, long,value_parser = is_dir, default_value=".")]
    pub out: String,
}
