use super::{
    delta::Delta,
    number::{self, Number},
    value::Value,
    EncodedOption,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedOption {
    pub number: Number,
    pub values: Vec<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    EmptyOptions,
    Number(number::Error),
}

impl DecodedOption {
    pub fn encode(self, delta_sum: Delta) -> Vec<u8> {
        let number = self.number;
        let mut values = self.values.into_iter().filter(Value::is_bytes);

        match values.next() {
            Some(first_value) => {
                let mut result =
                    vec![EncodedOption::new(number.encode(delta_sum), first_value).encode()];

                result.extend(values.map(|x| EncodedOption::new(Delta::repeating(), x).encode()));

                result.into_iter().flatten().collect()
            }
            None => EncodedOption::new(number.encode(delta_sum), Value::Empty).encode(),
        }
    }

    pub fn parse(input: &[EncodedOption]) -> Result<(&[EncodedOption], Self), Error> {
        let mut options = input.iter();

        let Some(head) = options.next() else {
            return Err(Error::EmptyOptions);
        };

        let number = Number::decode(*head.delta())?;
        let mut values = vec![head.value().clone()];

        values.extend(
            options
                .take_while(|o| o.delta().is_repeating())
                .map(|o| o.value().clone()),
        );

        Ok((&input[values.len()..], Self { number, values }))
    }
}

impl From<number::Error> for Error {
    fn from(value: number::Error) -> Self {
        Self::Number(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{number, DecodedOption, Delta, EncodedOption, Error, Number, Value};

    #[rstest]
    #[case(
        DecodedOption{ number: Number::from_value(2).unwrap(), values: vec![] },
        Delta::from_value(0),
        &[0b0010_0000]
    )]
    #[case(
        DecodedOption{ number: Number::from_value(2).unwrap(), values: vec![] },
        Delta::from_value(1),
        &[0b0001_0000]
    )]
    #[case(
        DecodedOption{ number: Number::from_value(2).unwrap(), values: vec![] },
        Delta::from_value(0),
        &[0b0010_0000]
    )]
    #[case(
        DecodedOption{ number: Number::from_value(2).unwrap(), values: vec![Value::from_str("a").unwrap()] },
        Delta::from_value(0),
        &[0b0010_0001, 97]
    )]
    #[case(
        DecodedOption{ 
            number: Number::from_value(2).unwrap(), 
            values: vec![
                Value::from_str("a").unwrap(),
                Value::from_str("bc").unwrap(),
            ] 
        },
        Delta::from_value(0),
        &[0b0010_0001, 97, 0b0000_0010, 98, 99]
    )]
    fn encode(
        #[case] decoded_option: DecodedOption,
        #[case] delta_sum: Delta,
        #[case] expected: &[u8],
    ) {
        assert_eq!(expected, decoded_option.encode(delta_sum))
    }

    #[rstest]
    #[case(&[], &[], Err(Error::EmptyOptions))]
    #[case(
        &[EncodedOption::new(Delta::from_value(0), Value::from_str("a").unwrap())], 
        [].as_ref(),
        Err(Error::Number(number::Error::Reserved(Delta::from_value(0))))
    )]
    #[case(
        &[EncodedOption::new(Delta::from_value(2), Value::from_str("a").unwrap())], 
        [].as_ref(),
        Ok(DecodedOption{
            number: Number::from_value(2).unwrap(), 
            values: vec![Value::from_str("a").unwrap()]
        })
    )]
    #[case(
        &[
            EncodedOption::new(Delta::from_value(2), Value::from_str("a").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("b").unwrap()),
        ], 
        [].as_ref(),
        Ok(DecodedOption{
            number: Number::from_value(2).unwrap(), 
            values: vec![
                Value::from_str("a").unwrap(),
                Value::from_str("b").unwrap(),
            ]
        })
    )]
    #[case(
        &[
            EncodedOption::new(Delta::from_value(2), Value::from_str("a").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("b").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("c").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("d").unwrap()),
        ], 
        [].as_ref(),
        Ok(DecodedOption{
            number: Number::from_value(2).unwrap(), 
            values: vec![
                Value::from_str("a").unwrap(),
                Value::from_str("b").unwrap(),
                Value::from_str("c").unwrap(),
                Value::from_str("d").unwrap(),
            ]
        })
    )]
    #[case(
        &[
            EncodedOption::new(Delta::from_value(2), Value::from_str("a").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("b").unwrap()),
            EncodedOption::new(Delta::from_value(4), Value::from_str("c").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("d").unwrap()),
        ], 
        &[
            EncodedOption::new(Delta::from_value(4), Value::from_str("c").unwrap()),
            EncodedOption::new(Delta::from_value(0), Value::from_str("d").unwrap()),
        ],
        Ok(DecodedOption{
            number: Number::from_value(2).unwrap(), 
            values: vec![
                Value::from_str("a").unwrap(),
                Value::from_str("b").unwrap(),
            ]
        })
    )]
    fn parse(
        #[case] input: &[EncodedOption],
        #[case] expected_rest: &[EncodedOption],
        #[case] expected: Result<DecodedOption, Error>,
    ) {
        assert_eq!(
            expected.map(|value| (expected_rest, value)),
            DecodedOption::parse(input)
        );
    }
}
