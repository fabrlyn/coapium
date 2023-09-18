use crate::codec::parsing::many0;

use super::{
    decoded_option::{self, DecodedOption},
    encoded_option, EncodedOption,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedOptions {
    options: Vec<DecodedOption>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    EncodedOption(encoded_option::Error),
    DecodedOption(decoded_option::Error),
}

impl DecodedOptions {
    pub fn decode(encoded_options: Vec<EncodedOption>) -> Result<Self, Error> {
        let mut input: &[EncodedOption] = &encoded_options;

        let mut options = vec![];

        while !input.is_empty() {
            let (rest, option) = DecodedOption::parse(&input)?;

            input = rest;
            options.push(option);
        }

        Ok(Self { options })
    }

    pub fn decoded_options(self) -> impl Iterator<Item = DecodedOption> {
        self.options.into_iter()
    }

    pub fn parse(bytes: &[u8]) -> Result<(&[u8], Self), Error> {
        let (bytes, options) = many0(EncodedOption::parse)(bytes)?;
        Self::decode(options).map(|options| (bytes, options))
    }
}

impl From<encoded_option::Error> for Error {
    fn from(value: encoded_option::Error) -> Self {
        Self::EncodedOption(value)
    }
}

impl From<decoded_option::Error> for Error {
    fn from(value: decoded_option::Error) -> Self {
        Self::DecodedOption(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        super::decoded_option,
        super::number::{self, Number},
        super::Delta,
        super::Value,
        DecodedOption, DecodedOptions, EncodedOption, Error,
    };

    #[rstest]
    #[case(vec![], Ok(DecodedOptions { options: vec![] }))]
    #[case(
        vec![EncodedOption::new(Delta::from_value(0), Value::from_str("a").unwrap())],
        Err(Error::DecodedOption(decoded_option::Error::Number(number::Error::Reserved(Delta::from_value(0)))))
    )]
    #[case(
        vec![EncodedOption::new(Delta::from_value(11), Value::from_str("a").unwrap())],
        Ok(DecodedOptions { 
            options: vec![DecodedOption {number: Number::from_value(11).unwrap(), 
            values: vec![Value::from_str("a").unwrap()]}] 
        })
    )]
    #[case(
        vec![
            EncodedOption::new(Delta::from_value(11), Value::from_str("a").unwrap()),
            EncodedOption::new(Delta::from_value(15), Value::from_str("b").unwrap()),
        ],
        Ok(DecodedOptions { 
            options: vec![
                DecodedOption {
                    number: Number::from_value(11).unwrap(), 
                    values: vec![Value::from_str("a").unwrap()]
                },
                DecodedOption {
                    number: Number::from_value(15).unwrap(), 
                    values: vec![Value::from_str("b").unwrap()]
                }
            ] 
        })
    )]
    fn decode(#[case] input: Vec<EncodedOption>, #[case] expected: Result<DecodedOptions, Error>) {
        assert_eq!(expected, DecodedOptions::decode(input));
    }

    #[rstest]
    #[case(&[], &[], Ok(DecodedOptions{ options: vec![] }))]
    #[case(&[0b1111_0001, 97], &[0b1111_0001, 97], Ok(DecodedOptions { options: vec![] }))]
    #[case(&[0b0001_1111, 97], &[0b0001_1111, 97], Ok(DecodedOptions { options: vec![] }))]
    #[case(&[0b1111_1111, 97], &[0b1111_1111, 97], Ok(DecodedOptions { options: vec![] }))]
    #[case(&[0b1011_0001, 97, 98], &[98], Ok(DecodedOptions { options: vec![DecodedOption{ number: Number::from_value(11).unwrap(), values: vec![Value::from_str("a").unwrap()] }] }))]
    fn parse(
        #[case] bytes: &[u8],
        #[case] expected_rest: &[u8],
        #[case] expected: Result<DecodedOptions, Error>,
    ) {
        assert_eq!(
            expected.map(|v| (expected_rest, v)),
            DecodedOptions::parse(bytes)
        )
    }
}
