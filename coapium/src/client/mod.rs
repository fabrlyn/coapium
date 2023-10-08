pub mod url;

use crate::{
    codec::{
        self,
        option::{UriHost, UriPath, UriPort, UriQuery},
    },
    protocol::{
        reliability::Reliability,
        response::{self, Response},
        transmission_parameters::{ConfirmableParameters, NonConfirmableParameters},
    },
};

use self::url::Url;

pub trait RequestBuilder {
    fn port(self, port: UriPort) -> Self;
    fn host(self, host: UriHost) -> Self;
    fn path(self, path: UriPath) -> Self;
    fn query_parameter(self, query: UriQuery) -> Self;
}

#[derive(Debug)]
pub enum ReliabilityBuilder {
    Confirmable(),
    NonConfirmable(),
}

impl Default for ReliabilityBuilder {
    fn default() -> Self {
        Self::Confirmable()
    }
}

#[derive(Debug, Default)]
pub struct GetRequestBuilder {
    host: Option<UriHost>,
    path: Option<UriPath>,
    port: Option<UriPort>,
    query_parameter: Vec<UriQuery>,
    reliability: Option<Reliability>,
}

pub fn get() -> GetRequestBuilder {
    GetRequestBuilder::new()
}

impl GetRequestBuilder {
    pub fn url(_url: Url) -> Self {
        Self::new()
    }

    pub fn host(mut self, host: UriHost) -> Self {
        self.host = Some(host);
        self
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn path(mut self, path: UriPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn port(mut self, uri_port: UriPort) -> Self {
        self.port = Some(uri_port);
        self
    }

    pub fn query_parameter(mut self, query_parameter: UriQuery) -> Self {
        self.query_parameter.push(query_parameter);

        self
    }

    pub fn confirmable(mut self, confirmable_parameters: ConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::Confirmable(confirmable_parameters));
        self
    }

    pub fn non_confirmable(mut self, non_confirmable_parameters: NonConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::NonConfirmable(non_confirmable_parameters));
        self
    }
}

#[derive(Debug, Default)]
pub struct PostRequestBuilder {
    host: Option<UriHost>,
    path: Option<UriPath>,
    port: Option<UriPort>,
    query_parameter: Vec<UriQuery>,
    reliability: Option<Reliability>,
}

impl PostRequestBuilder {
    pub fn url(_url: Url) -> Self {
        Self::new()
    }

    pub fn host(mut self, host: UriHost) -> Self {
        self.host = Some(host);
        self
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn path(mut self, path: UriPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn port(mut self, uri_port: UriPort) -> Self {
        self.port = Some(uri_port);
        self
    }

    pub fn query_parameter(mut self, query_parameter: UriQuery) -> Self {
        self.query_parameter.push(query_parameter);

        self
    }

    pub fn confirmable(mut self, confirmable_parameters: ConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::Confirmable(confirmable_parameters));
        self
    }

    pub fn non_confirmable(mut self, non_confirmable_parameters: NonConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::NonConfirmable(non_confirmable_parameters));
        self
    }
}

#[derive(Debug, Default)]
pub struct PutRequestBuilder {
    host: Option<UriHost>,
    path: Option<UriPath>,
    port: Option<UriPort>,
    query_parameter: Vec<UriQuery>,
    reliability: Option<Reliability>,
}

impl PutRequestBuilder {
    pub fn url(_url: Url) -> Self {
        Self::new()
    }

    pub fn host(mut self, host: UriHost) -> Self {
        self.host = Some(host);
        self
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn path(mut self, path: UriPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn port(mut self, uri_port: UriPort) -> Self {
        self.port = Some(uri_port);
        self
    }

    pub fn query_parameter(mut self, query_parameter: UriQuery) -> Self {
        self.query_parameter.push(query_parameter);

        self
    }

    pub fn confirmable(mut self, confirmable_parameters: ConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::Confirmable(confirmable_parameters));
        self
    }

    pub fn non_confirmable(mut self, non_confirmable_parameters: NonConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::NonConfirmable(non_confirmable_parameters));
        self
    }
}

#[derive(Debug, Default)]
pub struct DeleteRequestBuilder {
    host: Option<UriHost>,
    path: Option<UriPath>,
    port: Option<UriPort>,
    query_parameter: Vec<UriQuery>,
    reliability: Option<Reliability>,
}

impl DeleteRequestBuilder {
    pub fn url(_url: Url) -> Self {
        Self::new()
    }

    pub fn host(mut self, host: UriHost) -> Self {
        self.host = Some(host);
        self
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn path(mut self, path: UriPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn port(mut self, uri_port: UriPort) -> Self {
        self.port = Some(uri_port);
        self
    }

    pub fn query_parameter(mut self, query_parameter: UriQuery) -> Self {
        self.query_parameter.push(query_parameter);

        self
    }

    pub fn confirmable(mut self, confirmable_parameters: ConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::Confirmable(confirmable_parameters));
        self
    }

    pub fn non_confirmable(mut self, non_confirmable_parameters: NonConfirmableParameters) -> Self {
        self.reliability = Some(Reliability::NonConfirmable(non_confirmable_parameters));
        self
    }
}

impl RequestBuilder for GetRequestBuilder {
    fn port(self, port: UriPort) -> Self {
        self.port(port)
    }

    fn host(self, host: UriHost) -> Self {
        self.host(host)
    }

    fn path(self, path: UriPath) -> Self {
        self.path(path)
    }

    fn query_parameter(self, query: UriQuery) -> Self {
        self.query_parameter(query)
    }
}

impl RequestBuilder for PostRequestBuilder {
    fn port(self, port: UriPort) -> Self {
        self.port(port)
    }

    fn host(self, host: UriHost) -> Self {
        self.host(host)
    }

    fn path(self, path: UriPath) -> Self {
        self.path(path)
    }

    fn query_parameter(self, query: UriQuery) -> Self {
        self.query_parameter(query)
    }
}

impl RequestBuilder for PutRequestBuilder {
    fn port(self, port: UriPort) -> Self {
        self.port(port)
    }

    fn host(self, host: UriHost) -> Self {
        self.host(host)
    }

    fn path(self, path: UriPath) -> Self {
        self.path(path)
    }

    fn query_parameter(self, query: UriQuery) -> Self {
        self.query_parameter(query)
    }
}

impl RequestBuilder for DeleteRequestBuilder {
    fn port(self, port: UriPort) -> Self {
        self.port(port)
    }

    fn host(self, host: UriHost) -> Self {
        self.host(host)
    }

    fn path(self, path: UriPath) -> Self {
        self.path(path)
    }

    fn query_parameter(self, query: UriQuery) -> Self {
        self.query_parameter(query)
    }
}

// TODO: Investigate how this could be incorporated deeper into the library.
// This might be fine, but needs a look.
#[derive(Debug)]
pub enum PingError {
    UnexpectedResponse(Response),
    AcknowledgementTimeout,
    Codec(codec::Error),
    Timeout,
}

pub fn into_ping_result(result: Result<Response, response::Error>) -> Result<(), PingError> {
    match result {
        Ok(response) => Err(PingError::UnexpectedResponse(response)),
        Err(error) => match error {
            response::Error::AcknowledgementTimeout => Err(PingError::AcknowledgementTimeout),
            response::Error::Codec(error) => Err(PingError::Codec(error)),
            response::Error::Reset => Ok(()),
            response::Error::Timeout => Err(PingError::Timeout),
        },
    }
}
