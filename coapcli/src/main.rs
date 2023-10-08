mod cli;
mod common;
mod delete;
mod get;
mod post;
mod put;
mod ping;

use std::error::Error;

use cli::Cli;

pub fn main() -> Result<(), Box<dyn Error>> {
    Cli::run()
}
