use std::fmt::Display;

use url::Url;

use super::option::{UriHost, UriPort};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scheme {
    Coap,
    Coaps,
}

impl Scheme {
    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "coap" => Some(Self::Coap),
            "coaps" => Some(Self::Coaps),
            _ => None,
        }
    }
}

impl TryFrom<&str> for Scheme {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Scheme::from_value(value).ok_or(())
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scheme::Coap => f.write_str("coap"),
            Scheme::Coaps => f.write_str("coaps"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Port {
    value: u16,
}

impl Port {
    pub fn from_value(value: u16) -> Self {
        Port { value }
    }

    pub fn value(&self) -> u16 {
        self.value
    }
}

impl Default for Port {
    fn default() -> Self {
        Self { value: 5683 }
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        u16::fmt(&self.value, f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    segments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    value: String,
}

impl Query {
    pub fn parameters(&self) -> Vec<QueryParameter> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueryParameter {
    value: String,
}

impl QueryParameter {
    pub fn as_key_value(&self) -> Option<KeyValueParameter> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyValueParameter {
    key: String,
    value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub scheme: Scheme,
    pub host: UriHost,
    pub port: Option<UriPort>,
}

impl Endpoint {
    pub fn from_str(value: &str) -> Result<Self, Error> {
        Self::try_from(value)
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?}://{:?}:{:?}",
            self.scheme, self.host, self.port
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Scheme,
    Host,
    Port,
    Format,
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        match value {
            url::ParseError::EmptyHost => Error::Host,
            url::ParseError::IdnaError => Error::Host,
            url::ParseError::InvalidPort => Error::Port,
            url::ParseError::InvalidIpv4Address => Error::Host,
            url::ParseError::InvalidIpv6Address => Error::Host,
            url::ParseError::InvalidDomainCharacter => Error::Host,
            url::ParseError::RelativeUrlWithoutBase => Error::Format,
            url::ParseError::RelativeUrlWithCannotBeABaseBase => Error::Format,
            url::ParseError::SetHostOnCannotBeABaseUrl => Error::Format,
            url::ParseError::Overflow => Error::Format,
            _ => Error::Format,
        }
    }
}

impl TryFrom<&str> for Endpoint {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(value)?;
        let scheme = Scheme::from_value(url.scheme()).ok_or(Error::Scheme)?;
        let host = url
            .host_str()
            .unwrap()
            .try_into()
            .map_err(|_| Error::Host)?;
        let port = url.port().map(UriPort::from_u16);

        Ok(Self { scheme, host, port })
    }
}
