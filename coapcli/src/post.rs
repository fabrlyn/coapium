use std::{
    error::Error,
    io::{stdin, IsTerminal, Read},
};

use clap::Args;
use coapium::{
    client::url::Url,
    codec::{option::ContentFormat, MediaType, Payload},
    synchronous::{post, post_payload},
};

use crate::common::{parse_content_format, parse_url};

#[derive(Clone, Args, Debug)]
pub struct Post {
    #[arg(long, value_parser = parse_url)]
    url: Url,

    #[arg(long, num_args(0..=1))]
    payload: Option<Option<String>>,

    #[arg(long, value_parser = parse_content_format)]
    content_format: Option<ContentFormat>,
}

impl Post {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let payload = self.payload()?;

        let response = if payload.is_empty() {
            post(self.url)
        } else {
            post_payload(self.url.clone(), self.content_format(), payload)
        }
        .map_err(|e| format!("{:?}", e))?;

        println!("-- Response code --\n{:?}", response.response_code);
        if let Ok(payload) = String::from_utf8(response.payload.value().to_vec()) {
            println!("-- Payload -- \n{payload}");
        } else {
            println!("-- Payload -- \n{:?}", response.payload.value());
        }

        Ok(())
    }

    fn stdin_payload() -> Result<Payload, Box<dyn Error>> {
        let Ok(payload) = std::io::stdin().bytes().collect::<Result<Vec<_>, _>>() else {
            return Err("failed to read stdin".into());
        };

        Ok(Payload::from_value(payload))
    }

    fn payload(&self) -> Result<Payload, Box<dyn Error>> {
        if !stdin().is_terminal() {
            return Self::stdin_payload();
        }

        match &self.payload {
            Some(Some(payload)) => Ok(Payload::from_value(payload.clone().into_bytes())),
            Some(None) => Self::stdin_payload(),
            None => Ok(Payload::empty()),
        }
    }

    fn content_format(&self) -> ContentFormat {
        self.content_format
            .clone()
            .unwrap_or(MediaType::CharsetUtf8.into())
    }
}
