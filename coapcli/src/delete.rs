use std::error::Error;

use clap::Args;
use coapium::{asynchronous::delete, client::url::Url};

use crate::common::parse_url;

#[derive(Clone, Args, Debug)]
pub struct Delete {
    #[arg(long, value_parser = parse_url)]
    url: Url,
}

impl Delete {
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let response = delete(self.url).await.unwrap();

        println!("-- Response code --\n{:?}", response.response_code);
        if let Ok(payload) = String::from_utf8(response.payload.value().to_vec()) {
            println!("-- Payload -- \n{payload}");
        } else {
            println!("-- Payload -- \n{:?}", response.payload.value());
        }

        Ok(())
    }
}
