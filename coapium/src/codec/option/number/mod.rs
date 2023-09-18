pub mod cache_key;
pub mod class;
pub mod forward;

use self::{class::Class, forward::Forward};

use super::delta::Delta;

const RESERVED: [Delta; 5] = [
    Delta::from_value(0),
    Delta::from_value(128),
    Delta::from_value(132),
    Delta::from_value(136),
    Delta::from_value(140),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Number {
    pub class: Class,
    pub forward: Forward,
    pub value: Delta,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Reserved(Delta),
}

impl Number {
    pub fn decode(delta: Delta) -> Result<Self, Error> {
        if RESERVED.contains(&delta) {
            return Err(Error::Reserved(delta));
        }

        let flags = (delta.value() & (u8::MAX as u16)) as u8;

        let class = Class::decode(flags);
        let forward = Forward::decode(flags);

        Ok(Self {
            class,
            forward,
            value: delta,
        })
    }

    pub fn encode(self, delta_sum: Delta) -> Delta {
        self.value - delta_sum
    }

    pub fn from_value(value: u16) -> Result<Self, Error> {
        Self::decode(Delta::from_value(value))
    }

    pub fn from_value_or_panic(value: u16) -> Self {
        match Self::from_value(value) {
            Ok(number) => number,
            Err(_) => panic!("Invalid Number value"),
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::option::{
        delta::Delta,
        number::{class::Class, Error, Number},
    };

    #[rstest]
    #[case(Number::from_value(5).unwrap(), Number::from_value(15).unwrap(), Ordering::Less)]
    #[case(Number::from_value(15).unwrap(), Number::from_value(5).unwrap(), Ordering::Greater)]
    #[case(Number::from_value(65000).unwrap(), Number::from_value(5).unwrap(), Ordering::Greater)]
    fn ordering(#[case] a: Number, #[case] b: Number, #[case] expected: Ordering) {
        assert_eq!(expected, a.cmp(&b))
    }

    #[rstest]
    fn sort() {
        let mut numbers = vec![
            Number::from_value(65000).unwrap(),
            Number::from_value(5).unwrap(),
            Number::from_value(15).unwrap(),
        ];

        numbers.sort();

        assert_eq!(
            vec![
                Number::from_value(5).unwrap(),
                Number::from_value(15).unwrap(),
                Number::from_value(65000).unwrap(),
            ],
            numbers
        );
    }

    #[rstest]
    fn decode_encode() {
        for value in 0..=u16::MAX {
            if [0, 128, 132, 136, 140].contains(&value) {
                continue;
            }

            let decoded = Number::decode(Delta::from_value(value)).unwrap();
            let encoded = decoded.encode(Delta::repeating());

            let class = if value % 2 == 0 {
                Class::Elective
            } else {
                Class::Critical
            };

            assert_eq!(encoded.value(), value);
            assert_eq!(class, decoded.class);
        }
    }

    #[rstest]
    fn decode_reserved() {
        for value in [0, 128, 132, 136, 140] {
            let value = Delta::from_value(value);
            assert_eq!(Err(Error::Reserved(value)), Number::decode(value));
        }
    }

    #[rstest]
    #[case(Number::from_value(5).unwrap(), Delta::from_value(0), Delta::from_value(5))]
    #[case(Number::from_value(5).unwrap(), Delta::from_value(1), Delta::from_value(4))]
    fn encode(#[case] number: Number, #[case] delta_sum: Delta, #[case] expected: Delta) {
        assert_eq!(expected, number.encode(delta_sum))
    }

    #[rstest]
    fn from_value() {
        for value in 0..=u16::MAX {
            let delta = Delta::from_value(value);
            if [0, 128, 132, 136, 140].contains(&value) {
                continue;
            }

            assert_eq!(
                delta,
                Number::from_value(value)
                    .unwrap()
                    .encode(Delta::repeating())
            );
        }
    }

    #[rstest]
    fn from_value_reserved() {
        for value in [0, 128, 132, 136, 140] {
            let delta = Delta::from_value(value);
            assert_eq!(Err(Error::Reserved(delta)), Number::from_value(value));
        }
    }
}
