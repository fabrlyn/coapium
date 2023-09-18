use crate::codec::{
    option::{uri_host, uri_path, uri_port, UriHost, UriPath, UriPort, UriQuery},
    url::{Endpoint, Scheme},
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Scheme(String),
    Path(uri_path::Error),
    Host(uri_host::ValueError),
    Port(uri_port::DecodeError),
    Other(String), // TODO: Hopefully this can be removed since we should be able to our own parsing with our primitives
}

#[derive(Clone, Debug, PartialEq)]
pub struct Url {
    pub scheme: Scheme,
    pub host: UriHost,
    pub port: Option<UriPort>,
    pub path: UriPath,
    pub query: UriQuery,
}

impl From<Url> for Endpoint {
    fn from(value: Url) -> Self {
        Endpoint {
            scheme: value.scheme,
            host: value.host,
            port: value.port,
        }
    }
}

impl From<uri_host::ValueError> for Error {
    fn from(value: uri_host::ValueError) -> Self {
        Self::Host(value)
    }
}

impl From<uri_path::Error> for Error {
    fn from(value: uri_path::Error) -> Self {
        Self::Path(value)
    }
}

impl TryFrom<&str> for Url {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        url::Url::parse(value)
            .map_err(|e| Error::Other(e.to_string()))?
            .try_into()
    }
}

impl TryFrom<String> for Url {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<url::Url> for Url {
    type Error = Error;

    fn try_from(value: url::Url) -> Result<Self, Self::Error> {
        let query = value
            .query()
            .map(|query| {
                query
                    .split("&")
                    .fold(UriQuery::new(), |mut acc, parameter| {
                        acc.add_value(parameter); // TODO: This does not handle already url encoded query parameters
                        acc
                    })
            })
            .unwrap_or(UriQuery::new());

        Ok(Self {
            scheme: value
                .scheme()
                .try_into()
                .map_err(|_| Error::Scheme(value.scheme().to_owned()))?,
            host: value.host_str().unwrap_or("").try_into()?, // TODO: This does not handle already url encoded hosts
            port: value.port().map(|p| p.into()),
            path: value.path().try_into()?, // TODO: This does not handle already url encoded paths
            query,
        })
    }
}
