use crate::codec::option::{ContentFormat, UriHost, UriPath, UriPort, UriQuery};
use crate::codec::options;
use crate::codec::{option::Number, Options};

#[derive(Clone, Debug, PartialEq)]
pub struct PostOptions {
    options: Options,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Options(options::Error),
    Unrecognized(Number),
}

impl PostOptions {
    pub fn encode(self) -> Vec<u8> {
        self.options.encode()
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

    pub fn new() -> Self {
        Self {
            options: Options::new(),
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let (bytes, options) = Options::parse(bytes)?;
        Ok((bytes, PostOptions::from_options(options)?))
    }

    fn recognized_options() -> Vec<Number> {
        vec![
            ContentFormat::number(),
            UriHost::number(),
            UriPath::number(),
            UriPort::number(),
            UriQuery::number(),
        ]
    }

    pub fn set_content_format(&mut self, host: ContentFormat) {
        self.options.set_content_format(host)
    }

    pub fn set_uri_host(&mut self, host: UriHost) {
        self.options.set_uri_host(host)
    }

    pub fn set_uri_path(&mut self, path: UriPath) {
        self.options.set_uri_path(path)
    }

    pub fn set_uri_port(&mut self, port: UriPort) {
        self.options.set_uri_port(port)
    }

    pub fn set_uri_query(&mut self, path: UriQuery) {
        self.options.set_uri_query(path)
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

    use super::{ContentFormat, Options, PostOptions, UriHost, UriPath, UriPort, UriQuery};
    use crate::codec::MediaType;

    #[rstest]
    #[case(
        PostOptions { options: Options::new() }, 
        ContentFormat::from(MediaType::ApplicationXml),
        PostOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_content_format(MediaType::ApplicationXml.into());
                options 
           } 
        }
    )]
    fn set_content_format(
        #[case] mut post_options: PostOptions,
        #[case] content_format: ContentFormat,
        #[case] expected: PostOptions,
    ) {
        post_options.set_content_format(content_format);
        assert_eq!(expected, post_options)
    }

    #[rstest]
    #[case(
        PostOptions { options: Options::new() }, 
        UriHost::try_from("robertbarl.in").unwrap(),
        PostOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_host("robertbarl.in".try_into().unwrap());
                options 
           } 
        }
    )]
    fn set_uri_host(
        #[case] mut post_options: PostOptions,
        #[case] uri_host: UriHost,
        #[case] expected: PostOptions,
    ) {
        post_options.set_uri_host(uri_host);
        assert_eq!(expected, post_options)
    }

    #[rstest]
    #[case(
        PostOptions { options: Options::new() }, 
        UriPath::try_from("a/b/c").unwrap(),
        PostOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_path("a/b/c".try_into().unwrap());
                options 
           } 
        }
    )]
    fn set_uri_path(
        #[case] mut post_options: PostOptions,
        #[case] uri_path: UriPath,
        #[case] expected: PostOptions,
    ) {
        post_options.set_uri_path(uri_path);
        assert_eq!(expected, post_options)
    }

    #[rstest]
    #[case(
        PostOptions { options: Options::new() }, 
        1337.into(),
        PostOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_port(1337.into()); 
                options 
           } 
        }
    )]
    fn set_uri_port(
        #[case] mut post_options: PostOptions,
        #[case] uri_port: UriPort,
        #[case] expected: PostOptions,
    ) {
        post_options.set_uri_port(uri_port);
        assert_eq!(expected, post_options)
    }

    #[rstest]
    #[case(
        PostOptions { options: Options::new() }, 
        UriQuery::new(),
        PostOptions { 
            options: { 
                let mut options = Options::new(); 
                options.set_uri_query(UriQuery::new()); 
                options 
           } 
        }
    )]
    fn set_uri_query(
        #[case] mut post_options: PostOptions,
        #[case] uri_query: UriQuery,
        #[case] expected: PostOptions,
    ) {
        post_options.set_uri_query(uri_query);
        assert_eq!(expected, post_options)
    }
}
