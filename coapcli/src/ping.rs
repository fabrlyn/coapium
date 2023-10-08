use std::error::Error;

use clap::Args;
use coapium::{client::url::Url, synchronous::ping};

use crate::common::parse_url;

#[derive(Clone, Args, Debug)]
pub struct Ping {
    #[arg(long, value_parser = parse_url)]
    url: Url,
}

impl Ping {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        ping(self.url).unwrap();

        println!("-- Ping response --\n");

        Ok(())
    }
}
