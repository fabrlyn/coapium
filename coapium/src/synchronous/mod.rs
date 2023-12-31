pub mod client;
pub mod system;

use rand::{thread_rng, Rng};

use crate::{
    client::url::Url,
    codec::{
        message::{DeleteOptions, GetOptions, PostOptions, PutOptions},
        option::ContentFormat,
        Payload,
    },
    protocol::{
        delete::Delete,
        get::Get,
        new_request::NewRequest,
        ping::{self, Ping},
        post::Post,
        put::Put,
        reliability::Reliability,
        request::Method,
        response::{self, Response},
        transmission_parameters::{ConfirmableParameters, InitialRetransmissionFactor},
    },
    synchronous::client::Client,
};

pub fn default_parameters() -> ConfirmableParameters {
    ConfirmableParameters::new(
        Default::default(),
        Default::default(),
        initial_retransmission_factor(),
        Default::default(),
    )
}

pub fn default_reliability() -> Reliability {
    Reliability::Confirmable(default_parameters())
}

pub fn get(url: Url) -> Result<Response, response::Error> {
    request(Method::Get, url)
}

fn initial_retransmission_factor() -> InitialRetransmissionFactor {
    InitialRetransmissionFactor::new(thread_rng().gen_range(0.0..1.0)).unwrap()
}

pub fn ping(url: Url) -> Result<(), ping::Error> {
    Client::new(url.clone().into()).ping(Ping {
        confirmable_parameters: default_parameters(),
    })
}

pub fn post(url: Url) -> Result<Response, response::Error> {
    request(Method::Post, url)
}

pub fn post_payload(
    url: Url,
    content_format: ContentFormat,
    payload: Payload,
) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into());

    let reliability = default_reliability();

    let mut options = PostOptions::new();
    options.set_uri_path(url.path);
    options.set_uri_query(url.query);
    options.set_content_format(content_format);

    let request = NewRequest::Post(Post {
        options,
        reliability,
        payload,
    });

    client.execute(request)
}

pub fn put(url: Url) -> Result<Response, response::Error> {
    request(Method::Put, url)
}

pub fn put_payload(
    url: Url,
    content_format: ContentFormat,
    payload: Payload,
) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into());

    let reliability = default_reliability();

    let mut options = PutOptions::new();
    options.set_uri_path(url.path);
    options.set_uri_query(url.query);
    options.set_content_format(content_format);

    let request = NewRequest::Put(Put {
        options,
        reliability,
        payload,
    });

    client.execute(request)
}

pub fn delete(url: Url) -> Result<Response, response::Error> {
    request(Method::Delete, url)
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
