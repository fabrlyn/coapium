use crate::codec::{option::Number, Options};
use crate::codec::{
    option::{
        accept::Accept, proxy_scheme::ProxyScheme, proxy_uri::ProxyUri, uri_host::UriHost,
        uri_path::UriPath, uri_port::UriPort, uri_query::UriQuery, ETag,
    },
    options,
};

#[derive(Clone, Debug, PartialEq)]
pub struct GetOptions {
    options: Options,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Options(options::Error),
    Unrecognized(Number),
}

impl GetOptions {
    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let (bytes, options) = Options::parse(bytes)?;
        Ok((bytes, GetOptions::from_options(options)?))
    }

    pub fn new() -> Self {
        Self {
            options: Options::new(),
        }
    }

    pub fn set_uri_host(&mut self, host: UriHost) {
        self.options.set_uri_host(host)
    }

    pub fn set_uri_port(&mut self, port: UriPort) {
        self.options.set_uri_port(port)
    }

    pub fn set_uri_path(&mut self, path: UriPath) {
        self.options.set_uri_path(path)
    }

    pub fn set_uri_query(&mut self, path: UriQuery) {
        self.options.set_uri_query(path)
    }

    pub fn from_options(options: Options) -> Result<Self, Error> {
        if let Some(option) = options
            .options()
            .iter()
            .filter(|option| option.number().class.is_critical())
            .find(|option| !Self::recognized_options().contains(&option.number()))
        {
            return Err(Error::Unrecognized(option.number()));
        }

        Ok(Self { options })
    }

    fn recognized_options() -> Vec<Number> {
        vec![
            Accept::number(),
            ETag::number(),
            ProxyScheme::number(),
            ProxyUri::number(),
            UriHost::number(),
            UriPath::number(),
            UriPort::number(),
            UriQuery::number(),
        ]
    }

    pub fn encode(self) -> Vec<u8> {
        self.options.encode()
    }
}

impl From<options::Error> for Error {
    fn from(error: options::Error) -> Self {
        Self::Options(error)
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{GetOptions, Options, UriHost, UriPath, UriPort, UriQuery};

    #[rstest]
    #[case(
        GetOptions { options: Options::new() }, 
        UriHost::try_from("robertbarl.in").unwrap(),
        GetOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_host("robertbarl.in".try_into().unwrap());
                options 
           } 
        }
    )]
    fn set_uri_host(
        #[case] mut get_options: GetOptions,
        #[case] uri_host: UriHost,
        #[case] expected: GetOptions,
    ) {
        get_options.set_uri_host(uri_host);
        assert_eq!(expected, get_options)
    }

    #[rstest]
    #[case(
        GetOptions { options: Options::new() }, 
        UriPath::try_from("a/b/c").unwrap(),
        GetOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_path("a/b/c".try_into().unwrap());
                options 
           } 
        }
    )]
    fn set_uri_path(
        #[case] mut get_options: GetOptions,
        #[case] uri_path: UriPath,
        #[case] expected: GetOptions,
    ) {
        get_options.set_uri_path(uri_path);
        assert_eq!(expected, get_options)
    }

    #[rstest]
    #[case(
        GetOptions { options: Options::new() }, 
        1337.into(),
        GetOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_port(1337.into()); 
                options 
           } 
        }
    )]
    fn set_uri_port(
        #[case] mut get_options: GetOptions,
        #[case] uri_port: UriPort,
        #[case] expected: GetOptions,
    ) {
        get_options.set_uri_port(uri_port);
        assert_eq!(expected, get_options)
    }

    #[rstest]
    #[case(
        GetOptions { options: Options::new() }, 
        UriQuery::new(),
        GetOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_query(UriQuery::new()); 
                options 
           } 
        }
    )]
    fn set_uri_query(
        #[case] mut get_options: GetOptions,
        #[case] uri_query: UriQuery,
        #[case] expected: GetOptions,
    ) {
        get_options.set_uri_query(uri_query);
        assert_eq!(expected, get_options)
    }
}
