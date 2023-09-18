use std::error::Error;

use clap::Args;
use coapium::{synchronous::get, client::url::Url};

use crate::common::parse_url;

// TODO: There are two main ways of doing requests.
// Either assume that all the values are urlencoded already or not.
// Based on this there are two ways of consumer basic requests.
// Either supply the full url or supply the parts.
// This could of course be combined as well, by supplying the full url and using specific
// options to replace things in the url, like the port.
// `--url coap://localhost:1234/a/b/c --port 5678`

// TODO: supply a `get()` method for convenience, this method must do url encoding for everything.
// The default request building methods must also be url encoding by default.
// The option to use methods where the 'raw' string is used must exist.
// This is important for the cli application for example.

// TODO: The cli app can use the url encoded version for now and adapt later when I"ve done the 'raw' solution.

// NOTE: `reqwest` does url encoding on both path and query parameters. host cant contain url encoded characters

// TODO: Move endpoint from `codec` to `client` module.

// TODO: Avoid sending null query parameters, aka null

// TODO: payload, either via stdin or via flag --data or --data=some or --data="some text"

// TODO: content-type, either string name, number or default.

#[derive(Clone, Args, Debug)]
pub struct Get {
    #[arg(long, value_parser = parse_url)]
    url: Url,
}

impl Get {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let response = get(self.url).unwrap();

        println!("-- Response code --\n{:?}", response.response_code);
        if let Ok(payload) = String::from_utf8(response.payload.value().to_vec()) {
            println!("-- Payload -- \n{payload}");
        } else {
            println!("-- Payload -- \n{:?}", response.payload.value());
        }

        Ok(())
    }
}
