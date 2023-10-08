pub mod client;
pub mod system;

use crate::client::{into_ping_result, PingError};
use crate::codec::message::{DeleteOptions, GetOptions, PostOptions, PutOptions};
use crate::codec::option::ContentFormat;
use crate::codec::TokenLength;
use crate::codec::{Payload, Token};
use crate::protocol::delete::Delete;
use crate::protocol::get::Get;
use crate::protocol::new_request::NewRequest;
use crate::protocol::ping::Ping;
use crate::protocol::post::Post;
use crate::protocol::put::Put;
use crate::protocol::reliability::Reliability;
use crate::protocol::request::Method;
pub use crate::protocol::response;
use crate::protocol::transmission_parameters::{
    ConfirmableParameters, InitialRetransmissionFactor,
};
pub use client::Client;
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, RngCore, SeedableRng};

use crate::client::url::Url;

use self::response::Response;

fn default_reliability() -> Reliability {
    Reliability::Confirmable(default_parameters())
}

fn default_parameters() -> ConfirmableParameters {
    ConfirmableParameters::new(
        Default::default(),
        Default::default(),
        initial_retransmission_factor(),
        Default::default(),
    )
}

pub async fn delete(url: Url) -> Result<Response, response::Error> {
    request(Method::Delete, url).await
}

pub async fn get(url: Url) -> Result<Response, response::Error> {
    request(Method::Get, url).await
}

fn initial_retransmission_factor() -> InitialRetransmissionFactor {
    InitialRetransmissionFactor::new(thread_rng().gen_range(0.0..1.0)).unwrap()
}

pub async fn ping(url: Url) -> Result<(), PingError> {
    let result = Client::new(url.clone().into())
        .await
        .execute(NewRequest::Ping(Ping {
            confirmable_parameters: default_parameters(),
        }))
        .await;

    into_ping_result(result)
}

pub async fn post(url: Url) -> Result<Response, response::Error> {
    request(Method::Post, url).await
}

pub async fn post_payload(
    url: Url,
    content_format: ContentFormat,
    payload: Payload,
) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into()).await;

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

    client.execute(request).await
}

pub async fn put(url: Url) -> Result<Response, response::Error> {
    request(Method::Put, url).await
}

pub async fn put_payload(
    url: Url,
    content_format: ContentFormat,
    payload: Payload,
) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into()).await;

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

    client.execute(request).await
}

pub async fn request(method: Method, url: Url) -> Result<Response, response::Error> {
    let client = Client::new(url.clone().into()).await;

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

    client.execute(request).await
}

// TODO: Source token from here
fn token() -> Token {
    let mut rng = StdRng::from_entropy();
    let mut bytes = [0; TokenLength::MAX as usize];
    rng.fill_bytes(&mut bytes);
    Token::from_value(bytes.to_vec()).unwrap()
}
