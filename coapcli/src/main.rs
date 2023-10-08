mod cli;
mod common;
mod delete;
mod get;
mod ping;
mod post;
mod put;

use std::error::Error;

use cli::Cli;

pub fn main() -> Result<(), Box<dyn Error>> {
    Cli::run()
}
