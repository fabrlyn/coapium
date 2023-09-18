use std::convert::identity;

use crate::codec::option;
use crate::codec::option::Delta;
use crate::codec::option::IfMatch;
use crate::codec::option::MaxAge;
use crate::codec::option::Option;
use crate::codec::option::UriPath;

use super::option::ContentFormat;
use super::option::UriHost;
use super::option::UriPort;
use super::option::UriQuery;
use super::{
    option::decoded_option::DecodedOption,
    option::decoded_options::{self, DecodedOptions},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    options: Vec<Option>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Option(option::Error),
    DecodedOptions(decoded_options::Error),
}

impl Options {
    pub fn content_format(&self) -> std::option::Option<&ContentFormat> {
        self.options.iter().find_map(|o| o.content_format())
    }

    pub fn decode(options: DecodedOptions) -> Result<Self, Error> {
        Ok(Self {
            options: options
                .decoded_options()
                .map(Self::decode_option)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(identity)
                .collect(),
        })
    }

    fn decode_option(option: DecodedOption) -> Result<std::option::Option<Option>, Error> {
        Option::decode(option).map_err(Into::into)
    }

    pub fn encode(mut self) -> Vec<u8> {
        self.options.sort_by_key(Option::number);
        self.options
            .into_iter()
            .fold(
                (Delta::from_value(0), vec![]),
                |(delta_sum, mut encoded_options), o| {
                    let number = o.number();
                    let encoded_option = o.encode(delta_sum);
                    encoded_options.push(encoded_option);
                    (number.value, encoded_options)
                },
            )
            .1
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn if_match(&self) -> std::option::Option<&IfMatch> {
        self.options.iter().find_map(|o| o.if_match())
    }

    pub fn new() -> Self {
        Self { options: vec![] }
    }

    pub fn max_age(&self) -> std::option::Option<&MaxAge> {
        self.options.iter().find_map(|o| o.max_age())
    }

    pub fn options(&self) -> &[Option] {
        &self.options
    }

    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let (bytes, options) = DecodedOptions::parse(bytes)?;

        Ok((bytes, Self::decode(options)?))
    }

    pub fn set_content_format(&mut self, content_format: ContentFormat) {
        match self.options.iter().position(|x| x.is_content_format()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::ContentFormat(content_format))
            }
            None => self.options.push(Option::ContentFormat(content_format)),
        }
    }

    pub fn set_if_match(&mut self, if_match: IfMatch) {
        match self.options.iter().position(|x| x.is_if_match()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::IfMatch(if_match))
            }
            None => self.options.push(Option::IfMatch(if_match)),
        }
    }

    pub fn set_max_age(&mut self, max_age: MaxAge) {
        match self.options.iter().position(|x| x.is_max_age()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::MaxAge(max_age))
            }
            None => self.options.push(Option::MaxAge(max_age)),
        }
    }

    pub fn set_uri_host(&mut self, host: UriHost) {
        match self.options.iter().position(|x| x.is_uri_host()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::UriHost(host))
            }
            None => self.options.push(Option::UriHost(host)),
        }
    }

    pub fn set_uri_path(&mut self, path: UriPath) {
        match self.options.iter().position(|x| x.is_uri_path()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::UriPath(path))
            }
            None => self.options.push(Option::UriPath(path)),
        }
    }

    pub fn set_uri_port(&mut self, port: UriPort) {
        match self.options.iter().position(|x| x.is_uri_port()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::UriPort(port))
            }
            None => self.options.push(Option::UriPort(port)),
        }
    }

    pub fn set_uri_query(&mut self, query: UriQuery) {
        match self.options.iter().position(|x| x.is_uri_query()) {
            Some(position) => {
                self.options.swap_remove(position);
                self.options.push(Option::UriQuery(query))
            }
            None => self.options.push(Option::UriQuery(query)),
        }
    }

    pub fn uri_host(&self) -> std::option::Option<&UriHost> {
        self.options.iter().find_map(|o| o.uri_host())
    }

    pub fn uri_path(&self) -> std::option::Option<&UriPath> {
        self.options.iter().find_map(|o| o.uri_path())
    }

    pub fn uri_port(&self) -> std::option::Option<&UriPort> {
        self.options.iter().find_map(|o| o.uri_port())
    }

    pub fn uri_query(&self) -> std::option::Option<&UriQuery> {
        self.options.iter().find_map(|o| o.uri_query())
    }
}

impl From<decoded_options::Error> for Error {
    fn from(value: decoded_options::Error) -> Self {
        Self::DecodedOptions(value)
    }
}

impl From<option::Error> for Error {
    fn from(value: option::Error) -> Self {
        Self::Option(value)
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::option::{
        uri_host, ContentFormat, Delta, IfMatch, MaxAge, UriHost, UriPath, UriQuery, Value,
    };
    use crate::codec::MediaType;

    use super::{super::option, super::EncodedOption, DecodedOptions, Error, Option, Options};

    #[rstest]
    #[case(DecodedOptions::decode(vec![]).unwrap(), Ok(Options { options: vec![] }))]
    #[case(
        DecodedOptions::decode(
            vec![
                EncodedOption::new(
                    Delta::from_value(3),
                    Value::from_str("a").unwrap()
                ),
            ]
        )
        .unwrap(), 
        Ok(Options{ options: vec![Option::UriHost(UriHost::from_value("a").unwrap())] })
    )]
    #[case(
        DecodedOptions::decode(
            vec![
                EncodedOption::new(
                    Delta::from_value(3),
                    Value::from_str("a").unwrap()
                ),
                EncodedOption::new(
                    Delta::from_value(0),
                    Value::from_str("b").unwrap()
                ),
            ]
        )
        .unwrap(), 
        Err(Error::Option(option::Error::UriHost(uri_host::DecodeError::SingleValue)))
    )]
    fn decode(#[case] decoded_options: DecodedOptions, #[case] expected: Result<Options, Error>) {
        assert_eq!(expected, Options::decode(decoded_options))
    }

    #[rstest]
    #[case(Options::new(), vec![])]
    #[case(
        {
            let mut options = Options::new();
            options.set_uri_port(5432.into());
            options
        },
        vec![0b0111_0010, 21, 56]
    )]
    #[case(
        {
            let mut options = Options::new();
            options.set_uri_query("abc".try_into().unwrap());
            options.set_uri_host("127.0.0.1".try_into().unwrap());
            options.set_uri_path("a/b".try_into().unwrap());
            options.set_uri_port(5432.into());
            options
        },
        vec![
            0b0011_1001, 49, 50, 55, 46, 48, 46, 48, 46, 49,
            0b0100_0010, 21, 56,
            0b0100_0001, 97,
            0b0000_0001, 98,
            0b0100_0011, 97, 98, 99
        ]
    )]
    fn encode(#[case] options: Options, #[case] expected: Vec<u8>) {
        assert_eq!(expected, options.encode())
    }

    #[rstest]
    fn new() {
        assert_eq!(Vec::<u8>::new(), Options::new().encode());
    }

    #[rstest]
    #[case(Options::new(), &[])]
    #[case(
        {
            let mut options = Options::new();
            options.set_uri_path("a/b".try_into().unwrap());
            options
        }, 
        &[Option::UriPath(UriPath::from_value("a/b").unwrap())]
    )]
    fn get_options(#[case] options: Options, #[case] expected: &[Option]) {
        assert_eq!(expected, options.options())
    }

    #[rstest]
    #[case(&[], &[], Ok(Options { options: vec![] }))]
    #[case(&[0xff], &[0xff], Ok(Options { options: vec![] }))]
    #[case(&[0xff], &[0xff], Ok(Options { options: vec![] }))]
    #[case(
        &[0b1011_0001, 97, 98], 
        &[98], 
        Ok({
            let mut options = Options::new();
            options.set_uri_path("a".try_into().unwrap()); 
            options 
        })
    )]
    fn parse(
        #[case] input: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected: Result<Options, Error>,
    ) {
        assert_eq!(expected.map(|v| (expected_rest, v)), Options::parse(input))
    }

    #[rstest]
    #[case(Options::new(), &[Option::ContentFormat(MediaType::ApplicationJson.into())])]
    fn set_content_format_get_content_format(
        #[case] mut options: Options,
        #[case] expected: &[Option],
    ) {
        let content_format = ContentFormat::from(MediaType::ApplicationJson);

        options.set_content_format(content_format.clone());

        assert_eq!(Some(&content_format), options.content_format());
        assert_eq!(expected, options.options());
    }

    #[rstest]
    #[case(Options::new(), &[Option::IfMatch(IfMatch::from_values(vec![vec![1, 2]]).unwrap())])]
    fn set_if_match_get_if_match(#[case] mut options: Options, #[case] expected: &[Option]) {
        let if_match = IfMatch::from_values(vec![vec![1, 2]]).unwrap();

        options.set_if_match(if_match.clone());

        assert_eq!(Some(&if_match), options.if_match());
        assert_eq!(expected, options.options());
    }

    #[rstest]
    #[case(Options::new(), &[Option::MaxAge(13.into())])]
    fn set_max_age_get_max_age(#[case] mut options: Options, #[case] expected: &[Option]) {
        let max_age = MaxAge::from(13);

        options.set_max_age(max_age.clone());

        assert_eq!(Some(&max_age), options.max_age());
        assert_eq!(expected, options.options());
    }

    #[rstest]
    #[case(Options::new(), &[Option::UriHost(UriHost::try_from("robertbarl.in").unwrap())])]
    fn set_uri_host_get_uri_host(#[case] mut options: Options, #[case] expected: &[Option]) {
        let uri_host = UriHost::try_from("robertbarl.in").unwrap();

        options.set_uri_host(uri_host.clone());

        assert_eq!(Some(&uri_host), options.uri_host());
        assert_eq!(expected, options.options());
    }

    #[rstest]
    #[case(Options::new(), &[Option::UriPath(UriPath::from_value("a/b").unwrap())])]
    fn set_uri_path_get_uri_path(#[case] mut options: Options, #[case] expected: &[Option]) {
        let uri_path = UriPath::from_value("a/b").unwrap();

        options.set_uri_path(uri_path.clone());

        assert_eq!(Some(&uri_path), options.uri_path());
        assert_eq!(expected, options.options());
    }

    #[rstest]
    #[case(Options::new(), &[Option::UriQuery(UriQuery::new())])]
    fn set_uri_query_get_uri_query(#[case] mut options: Options, #[case] expected: &[Option]) {
        let uri_query = UriQuery::new();

        options.set_uri_query(uri_query.clone());

        assert_eq!(Some(&uri_query), options.uri_query());
        assert_eq!(expected, options.options());
    }
}
