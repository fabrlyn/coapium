mod cli;
mod common;
mod delete;
mod get;
mod post;
mod put;

use std::error::Error;

use cli::Cli;

// TODO: Should use the synchronous client when able
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    Cli::run().await
}
