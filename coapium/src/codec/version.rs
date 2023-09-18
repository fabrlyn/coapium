use Version::*;

/// Steps to shift to encode/decode a byte
const SHIFT: u8 = 6;

/// Numeric value of [`V1`](`Version::V1`)
const VERSION_1: u8 = 0b01;

/// The version number of the [`Message`](`crate::codec::Message`).
///
/// The version(`VER`) consists of a 2-bit value and are the first two bits in the first byte of the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
///  
/// ```markdown
/// 0                 
/// 0 1 2 3 4 5 6 7 8
/// +-+-+-+-+-+-+-+-+-
/// |Ver| T |  TKL  |
/// +-+-+-+-+-+-+-+-+-
/// ```
///
/// The current [specification](https://datatracker.ietf.org/doc/html/rfc7252#section-3) only supports one version, [`V1`](`Version::V1`).
///
/// All other version values are reserved for future use and **must** be ignored.
///
/// Any version value, expect [`V1`](`Version::V1`), will be treated as a parsing error and result in [`Unsupported`](`Error::Unsupported`).
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Version {
    /// Value(`0b01`) defined by [`VERSION_1`](`VERSION_1`)
    V1,
}

/// Possible errors when decoding [`Version`](`Version`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// The version value is reserved for future use and is not supported.
    ///
    /// Contains the unsupported value.
    Unsupported(u8),
}

impl Version {
    /// Parse the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub fn decode(byte: u8) -> Result<Self, Error> {
        match byte >> SHIFT {
            VERSION_1 => Ok(V1),
            unsupported => Err(Error::Unsupported(unsupported)),
        }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub fn encode(self) -> u8 {
        match self {
            V1 => self.value() << SHIFT,
        }
    }

    /// Returns `true` if the version is [`V1`](`Version::V1`)
    pub fn is_v1(&self) -> bool {
        match self {
            V1 => true,
        }
    }

    /// Get the numeric value of the version.
    ///
    /// Possible values:
    /// - [`VERSION_1`](`VERSION_1`)
    pub fn value(&self) -> u8 {
        match self {
            V1 => VERSION_1,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use std::ops::RangeInclusive;

    use super::{
        Error::{self, *},
        Version::{self, *},
    };

    #[rstest]
    #[case(0b00_000000..=0b00_111111, Err(Unsupported(0)))]
    #[case(0b01_000000..=0b01_111111, Ok(V1))]
    #[case(0b10_000000..=0b10_111111, Err(Unsupported(2)))]
    #[case(0b11_000000..=0b11_111111, Err(Unsupported(3)))]
    fn decode(#[case] inputs: RangeInclusive<u8>, #[case] expected: Result<Version, Error>) {
        for input in inputs {
            assert_eq!(expected, Version::decode(input));
        }
    }

    #[rstest]
    #[case(V1, 0b01_000000)]
    fn encode(#[case] input: Version, #[case] expected: u8) {
        assert_eq!(expected, input.encode());
    }

    #[rstest]
    #[case(V1, true)]
    fn is_v1(#[case] version: Version, #[case] expected: bool) {
        assert_eq!(expected, version.is_v1());
    }

    #[rstest]
    #[case(Version::V1, 1)]
    fn value(#[case] version: Version, #[case] expected: u8) {
        assert_eq!(expected, version.value())
    }
}
