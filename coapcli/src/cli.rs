use std::error::Error;

use clap::{command, Parser, Subcommand};

use crate::{delete::Delete, get::Get, post::Post, put::Put};

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    Get(Get),
    Post(Post),
    Put(Put),
    Delete(Delete),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

impl Cli {
    pub fn run() -> Result<(), Box<dyn Error>> {
        let cli = Cli::parse();

        match cli.commands {
            Commands::Get(command) => command.run(),
            Commands::Post(command) => command.run(),
            Commands::Put(command) => command.run(),
            Commands::Delete(command) => command.run(),
        }
    }
}
