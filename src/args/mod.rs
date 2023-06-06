use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(subcommand)]

    /// action to take
    pub action: Action,
}

#[derive(Parser, Debug)]
pub enum Action {
    #[clap(name = "generate")]
    Generate(Generate),
    #[clap(name = "fuzz")]
    Fuzz(Fuzz),
    #[clap(name = "mutate")]
    Mutate(Mutate),
    #[clap(name = "get_resources")]
    GetResources(GetResources),
}

#[derive(Parser, Debug)]
pub struct Generate {
    #[clap(short, long)]
    pub constraint: String,
    #[clap(short, long)]
    pub resource: String,
}

#[derive(Parser, Debug)]
pub struct Fuzz {}
#[derive(Parser, Debug)]
pub struct Mutate {}
#[derive(Parser, Debug)]
pub struct GetResources {}
