pub mod accept;
pub mod content_format;
pub mod decoded_option;
pub mod decoded_options;
pub mod delta;
pub mod delta_header;
pub mod encoded_option;
pub mod etag;
pub mod if_match;
pub mod if_none_match;
pub mod length;
pub mod length_header;
pub mod location_path;
pub mod location_query;
pub mod max_age;
pub mod number;
pub mod proxy_scheme;
pub mod proxy_uri;
pub mod size1;
pub mod uri_host;
pub mod uri_path;
pub mod uri_port;
pub mod uri_query;
pub mod value;

pub use accept::Accept;
pub use content_format::ContentFormat;
pub use decoded_option::DecodedOption;
pub use decoded_options::DecodedOptions;
pub use delta::Delta;
pub use delta_header::DeltaHeader;
pub use encoded_option::EncodedOption;
pub use etag::ETag;
pub use if_match::IfMatch;
pub use if_none_match::IfNoneMatch;
pub use length::Length;
pub use length_header::LengthHeader;
pub use location_path::LocationPath;
pub use location_query::LocationQuery;
pub use max_age::MaxAge;
pub use number::Number;
pub use proxy_scheme::ProxyScheme;
pub use proxy_uri::ProxyUri;
pub use size1::Size1;
pub use uri_host::UriHost;
pub use uri_path::UriPath;
pub use uri_port::UriPort;
pub use uri_query::UriQuery;
pub use value::Value;

// RFC:
// Not all options are defined for use with all methods and Response
// Codes.  The possible options for methods and Response Codes are
// defined in Sections 5.8 and 5.9, respectively.  In case an option is
// not defined for a Method or Response Code, it MUST NOT be included by
// a sender and MUST be treated like an unrecognized option by a
// recipient.
//
// - Upon reception, unrecognized options of class "elective" MUST be
// silently ignored.
//
// - Unrecognized options of class "critical" that occur in a
// Confirmable request MUST cause the return of a 4.02 (Bad Option)
// response.  This response SHOULD include a diagnostic payload
// describing the unrecognized option(s) (see Section 5.5.2).
//
// - Unrecognized options of class "critical" that occur in a
// Confirmable response, or piggybacked in an Acknowledgement, MUST
// cause the response to be rejected (Section 4.2).
//
// - Unrecognized options of class "critical" that occur in a Non-
// confirmable message MUST cause the message to be rejected
// (Section 4.3).
//
// Unsafe or Safe-to-Forward and NoCacheKey
//
// The definition of some options specifies that those options are
// repeatable.  An option that is repeatable MAY be included one or more
// times in a message.  An option that is not repeatable MUST NOT be
// included more than once in a message.
//
// If a message includes an option with more occurrences than the option
// is defined for, each supernumerary option occurrence that appears
// subsequently in the message MUST be treated like an unrecognized
// option (see Section 5.4.1).

#[derive(Clone, Debug, PartialEq)]
pub enum Option {
    Accept(Accept),
    ContentFormat(ContentFormat),
    ETag(ETag),
    IfMatch(IfMatch),
    IfNoneMatch(IfNoneMatch),
    LocationPath(LocationPath),
    LocationQuery(LocationQuery),
    MaxAge(MaxAge),
    ProxyScheme(ProxyScheme),
    ProxyUri(ProxyUri),
    Size1(Size1),
    UriHost(UriHost),
    UriPath(UriPath),
    UriPort(UriPort),
    UriQuery(UriQuery),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Accept(accept::Error),
    ContentFormat(content_format::Error),
    ETag(etag::Error),
    IfMatch(if_match::Error),
    IfNoneMatch(if_none_match::Error),
    LocationPath(location_path::Error),
    LocationQuery(location_query::Error),
    MaxAge(max_age::DecodeError),
    ProxyScheme(proxy_scheme::Error),
    ProxyUri(proxy_uri::Error),
    Size1(size1::Error),
    UriHost(uri_host::DecodeError),
    UriPath(uri_path::Error),
    UriPort(uri_port::DecodeError),
    UriQuery(uri_query::Error),
    Unrecognized(Number),
    Delta(delta::DecodeError),
    HeaderMissing,
    Length(length::DecodeError),
    Value(value::Error),
}

impl Option {
    pub fn content_format(&self) -> std::option::Option<&ContentFormat> {
        match self {
            Option::ContentFormat(content_format) => Some(content_format),
            _ => None,
        }
    }

    pub fn decode(option: DecodedOption) -> Result<std::option::Option<Self>, Error> {
        let option = match option.number {
            n if n == Accept::number() => Accept::decode(option.values).map(Self::Accept)?,
            n if n == ContentFormat::number() => {
                ContentFormat::decode(option.values).map(Self::ContentFormat)?
            }
            n if n == ETag::number() => ETag::decode(option.values).map(Self::ETag)?,
            n if n == IfMatch::number() => IfMatch::decode(option.values).map(Self::IfMatch)?,
            n if n == IfNoneMatch::number() => {
                IfNoneMatch::decode(option.values).map(Self::IfNoneMatch)?
            }
            n if n == LocationPath::number() => {
                LocationPath::decode(option.values).map(Self::LocationPath)?
            }
            n if n == LocationQuery::number() => {
                LocationQuery::decode(option.values).map(Self::LocationQuery)?
            }
            n if n == MaxAge::number() => MaxAge::decode(option.values).map(Self::MaxAge)?,
            n if n == ProxyScheme::number() => {
                ProxyScheme::decode(option.values).map(Self::ProxyScheme)?
            }
            n if n == ProxyUri::number() => ProxyUri::decode(option.values).map(Self::ProxyUri)?,
            n if n == Size1::number() => Size1::decode(option.values).map(Self::Size1)?,
            n if n == UriHost::number() => UriHost::decode(option.values).map(Self::UriHost)?,
            n if n == UriPath::number() => UriPath::decode(option.values).map(Self::UriPath)?,
            n if n == UriPort::number() => UriPort::decode(option.values).map(Self::UriPort)?,
            n if n == UriQuery::number() => UriQuery::decode(option.values).map(Self::UriQuery)?,
            _ => return Self::decode_unrecognized(option),
        };

        Ok(Some(option))
    }

    fn decode_unrecognized(option: DecodedOption) -> Result<std::option::Option<Self>, Error> {
        if option.number.class.is_critical() {
            Err(Error::Unrecognized(option.number))
        } else {
            Ok(None)
        }
    }

    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        match self {
            Option::Accept(o) => o.encode(delta_sum),
            Option::ContentFormat(o) => o.encode(delta_sum),
            Option::ETag(o) => o.encode(delta_sum),
            Option::IfMatch(o) => o.encode(delta_sum),
            Option::IfNoneMatch(o) => o.encode(delta_sum),
            Option::LocationPath(o) => o.encode(delta_sum),
            Option::LocationQuery(o) => o.encode(delta_sum),
            Option::MaxAge(o) => o.encode(delta_sum),
            Option::ProxyScheme(o) => o.encode(delta_sum),
            Option::ProxyUri(o) => o.encode(delta_sum),
            Option::Size1(o) => o.encode(delta_sum),
            Option::UriHost(o) => o.encode(delta_sum),
            Option::UriPath(o) => o.encode(delta_sum),
            Option::UriPort(o) => o.encode(delta_sum),
            Option::UriQuery(o) => o.encode(delta_sum),
        }
    }

    pub fn if_match(&self) -> std::option::Option<&IfMatch> {
        match self {
            Option::IfMatch(if_match) => Some(if_match),
            _ => None,
        }
    }

    pub fn is_content_format(&self) -> bool {
        match self {
            Option::ContentFormat(_) => true,
            _ => false,
        }
    }

    pub fn is_if_match(&self) -> bool {
        match self {
            Option::IfMatch(_) => true,
            _ => false,
        }
    }

    pub fn is_max_age(&self) -> bool {
        match self {
            Option::MaxAge(_) => true,
            _ => false,
        }
    }

    pub fn is_uri_host(&self) -> bool {
        match self {
            Option::UriHost(_) => true,
            _ => false,
        }
    }

    pub fn is_uri_path(&self) -> bool {
        match self {
            Option::UriPath(_) => true,
            _ => false,
        }
    }

    pub fn is_uri_port(&self) -> bool {
        match self {
            Option::UriPort(_) => true,
            _ => false,
        }
    }

    pub fn is_uri_query(&self) -> bool {
        match self {
            Option::UriQuery(_) => true,
            _ => false,
        }
    }

    pub fn number(&self) -> Number {
        match self {
            Option::Accept(_) => Accept::number(),
            Option::ContentFormat(_) => ContentFormat::number(),
            Option::ETag(_) => ETag::number(),
            Option::IfMatch(_) => IfMatch::number(),
            Option::IfNoneMatch(_) => IfNoneMatch::number(),
            Option::LocationPath(_) => LocationPath::number(),
            Option::LocationQuery(_) => LocationQuery::number(),
            Option::MaxAge(_) => MaxAge::number(),
            Option::ProxyScheme(_) => ProxyScheme::number(),
            Option::ProxyUri(_) => ProxyUri::number(),
            Option::Size1(_) => Size1::number(),
            Option::UriHost(_) => UriHost::number(),
            Option::UriPath(_) => UriPath::number(),
            Option::UriPort(_) => UriPort::number(),
            Option::UriQuery(_) => UriQuery::number(),
        }
    }

    pub fn max_age(&self) -> std::option::Option<&MaxAge> {
        match self {
            Option::MaxAge(max_age) => Some(max_age),
            _ => None,
        }
    }

    pub fn uri_host(&self) -> std::option::Option<&UriHost> {
        match self {
            Option::UriHost(uri_host) => Some(uri_host),
            _ => None,
        }
    }

    pub fn uri_path(&self) -> std::option::Option<&UriPath> {
        match self {
            Option::UriPath(uri_path) => Some(uri_path),
            _ => None,
        }
    }

    pub fn uri_port(&self) -> std::option::Option<&UriPort> {
        match self {
            Option::UriPort(uri_port) => Some(uri_port),
            _ => None,
        }
    }

    pub fn uri_query(&self) -> std::option::Option<&UriQuery> {
        match self {
            Option::UriQuery(uri_query) => Some(uri_query),
            _ => None,
        }
    }
}

impl From<accept::Error> for Error {
    fn from(value: accept::Error) -> Self {
        Self::Accept(value)
    }
}

impl From<content_format::Error> for Error {
    fn from(value: content_format::Error) -> Self {
        Self::ContentFormat(value)
    }
}

impl From<etag::Error> for Error {
    fn from(value: etag::Error) -> Self {
        Self::ETag(value)
    }
}

impl From<if_match::Error> for Error {
    fn from(value: if_match::Error) -> Self {
        Self::IfMatch(value)
    }
}

impl From<if_none_match::Error> for Error {
    fn from(value: if_none_match::Error) -> Self {
        Self::IfNoneMatch(value)
    }
}

impl From<location_path::Error> for Error {
    fn from(value: location_path::Error) -> Self {
        Self::LocationPath(value)
    }
}

impl From<location_query::Error> for Error {
    fn from(value: location_query::Error) -> Self {
        Self::LocationQuery(value)
    }
}

impl From<max_age::DecodeError> for Error {
    fn from(value: max_age::DecodeError) -> Self {
        Self::MaxAge(value)
    }
}

impl From<proxy_scheme::Error> for Error {
    fn from(value: proxy_scheme::Error) -> Self {
        Self::ProxyScheme(value)
    }
}

impl From<proxy_uri::Error> for Error {
    fn from(value: proxy_uri::Error) -> Self {
        Self::ProxyUri(value)
    }
}

impl From<size1::Error> for Error {
    fn from(value: size1::Error) -> Self {
        Self::Size1(value)
    }
}

impl From<uri_host::DecodeError> for Error {
    fn from(value: uri_host::DecodeError) -> Self {
        Self::UriHost(value)
    }
}

impl From<uri_path::Error> for Error {
    fn from(value: uri_path::Error) -> Self {
        Self::UriPath(value)
    }
}

impl From<uri_port::DecodeError> for Error {
    fn from(value: uri_port::DecodeError) -> Self {
        Self::UriPort(value)
    }
}

impl From<uri_query::Error> for Error {
    fn from(value: uri_query::Error) -> Self {
        Self::UriQuery(value)
    }
}

impl From<length::DecodeError> for Error {
    fn from(value: length::DecodeError) -> Self {
        Self::Length(value)
    }
}

impl From<delta::DecodeError> for Error {
    fn from(error: delta::DecodeError) -> Self {
        Self::Delta(error)
    }
}

impl From<value::Error> for Error {
    fn from(value: value::Error) -> Self {
        Self::Value(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        ContentFormat, Delta, EncodedOption, Option, UriHost, UriPath, UriPort, UriQuery, Value,
    };
    use crate::codec::MediaType;

    #[rstest]
    #[case(Option::ContentFormat(MediaType::ApplicationXml.into()), Some(ContentFormat::from(MediaType::ApplicationXml)))]
    #[case(Option::MaxAge(4567.into()), None)]
    fn content_format(
        #[case] option: Option,
        #[case] expected: std::option::Option<ContentFormat>,
    ) {
        assert_eq!(expected, option.content_format().map(|o| o.clone()))
    }

    #[rstest]
    #[case()]
    fn encode() {}

    #[rstest]
    #[case(Option::ContentFormat(MediaType::ApplicationJson.into()), true)]
    #[case(Option::MaxAge(4567.into()), false)]
    fn is_content_format(#[case] option: Option, #[case] expected: bool) {
        assert_eq!(expected, option.is_content_format())
    }

    #[rstest]
    #[case(Option::UriHost("robertbarl.in".try_into().unwrap()), true)]
    #[case(Option::MaxAge(4567.into()), false)]
    fn is_uri_host(#[case] option: Option, #[case] expected: bool) {
        assert_eq!(expected, option.is_uri_host())
    }

    #[rstest]
    #[case(Option::UriPath("a/b".try_into().unwrap()), true)]
    #[case(Option::MaxAge(4567.into()), false)]
    fn is_uri_path(#[case] option: Option, #[case] expected: bool) {
        assert_eq!(expected, option.is_uri_path())
    }

    #[rstest]
    #[case(Option::UriPort(4567.into()), true)]
    #[case(Option::MaxAge(4567.into()), false)]
    fn is_uri_port(#[case] option: Option, #[case] expected: bool) {
        assert_eq!(expected, option.is_uri_port())
    }

    #[rstest]
    #[case(Option::UriQuery(UriQuery::new()), true)]
    #[case(Option::MaxAge(4567.into()), false)]
    fn is_uri_query(#[case] option: Option, #[case] expected: bool) {
        assert_eq!(expected, option.is_uri_query())
    }

    #[rstest]
    #[case(&[0b0010_0010, 0b0000_0001, 0b0000_0010, 0b0000_0011], &[0b0000_0011], EncodedOption::new(Delta::from_value(2), Value::from_opaque(vec![1,2]).unwrap()))]
    #[case(&[0b0010_0000, 0b0000_0011], &[0b0000_0011], EncodedOption::new(Delta::from_value(2), Value::Empty))]
    fn parse(
        #[case] input: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected_output: EncodedOption,
    ) {
        assert_eq!(
            (expected_rest, expected_output),
            EncodedOption::parse(input).unwrap()
        );
    }

    #[rstest]
    #[case(Option::UriHost("robertbarl.in".try_into().unwrap()), Some(UriHost::try_from("robertbarl.in").unwrap()))]
    #[case(Option::MaxAge(4567.into()), None)]
    fn uri_host(#[case] option: Option, #[case] expected: std::option::Option<UriHost>) {
        assert_eq!(expected, option.uri_host().map(|o| o.clone()))
    }

    #[rstest]
    #[case(Option::UriPath("a/b".try_into().unwrap()), Some(UriPath::try_from("/a/b").unwrap()))]
    #[case(Option::MaxAge(4567.into()), None)]
    fn uri_path(#[case] option: Option, #[case] expected: std::option::Option<UriPath>) {
        assert_eq!(expected, option.uri_path().map(|o| o.clone()))
    }

    #[rstest]
    #[case(Option::UriPort(4567.into()), Some(UriPort::from(4567)))]
    #[case(Option::MaxAge(4567.into()), None)]
    fn uri_port(#[case] option: Option, #[case] expected: std::option::Option<UriPort>) {
        assert_eq!(expected, option.uri_port().map(|o| o.clone()))
    }

    #[rstest]
    #[case(Option::UriQuery(UriQuery::new()), Some(UriQuery::new()))]
    #[case(Option::MaxAge(4567.into()), None)]
    fn uri_query(#[case] option: Option, #[case] expected: std::option::Option<UriQuery>) {
        assert_eq!(expected, option.uri_query().map(|o| o.clone()))
    }
}
