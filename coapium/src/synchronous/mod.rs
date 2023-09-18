pub mod client;
pub mod system;

use rand::{thread_rng, Rng};

use crate::{
    client::url::Url,
    codec::{
        message::{DeleteOptions, GetOptions, PostOptions, PutOptions},
        Payload,
    },
    protocol::{
        delete::Delete,
        get::Get,
        new_request::NewRequest,
        post::Post,
        put::Put,
        reliability::Reliability,
        request::Method,
        response::{self, Response},
        transmission_parameters::{ConfirmableParameters, InitialRetransmissionFactor},
    },
    synchronous::client::Client,
};

pub fn get(url: Url) -> Result<Response, response::Error> {
    request(Method::Get, url)
}

pub fn request(method: Method, url: Url) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into());

    let reliability = default_reliability();

    let request = match method {
        Method::Get => {
            let mut options = GetOptions::new();
            options.set_uri_path(url.path);
            options.set_uri_query(url.query);

            NewRequest::Get(Get {
                options,
                reliability,
            })
        }
        Method::Post => {
            let mut options = PostOptions::new();
            options.set_uri_path(url.path);
            options.set_uri_query(url.query);

            NewRequest::Post(Post {
                options,
                reliability,
                payload: Payload::empty(),
            })
        }
        Method::Put => {
            let mut options = PutOptions::new();
            options.set_uri_path(url.path);
            options.set_uri_query(url.query);

            NewRequest::Put(Put {
                options,
                reliability,
                payload: Payload::empty(),
            })
        }
        Method::Delete => {
            let mut options = DeleteOptions::new();
            options.set_uri_path(url.path);
            options.set_uri_query(url.query);

            NewRequest::Delete(Delete {
                options,
                reliability,
            })
        }
    };

    client.execute(request)
}

fn initial_retransmission_factor() -> InitialRetransmissionFactor {
    InitialRetransmissionFactor::new(thread_rng().gen_range(0.0..1.0)).unwrap()
}

fn default_reliability() -> Reliability {
    Reliability::Confirmable(ConfirmableParameters::new(
        Default::default(),
        Default::default(),
        initial_retransmission_factor(),
        Default::default(),
    ))
}
