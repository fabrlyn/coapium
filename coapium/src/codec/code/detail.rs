/// Mask for decoding
const MASK: u8 = 0b000_11111;

const MAX: u8 = MASK;

/// The detail value of the [`Code`](`crate::codec::Code`) in a [`Message`](`crate::codec::Message`).
///
/// The detail(`detail`) consists of a 5-bit value and follows the class bits in the [`Code`](`crate::codec::Code`)
/// field in a [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
///  
/// ```markdown
/// 0
/// 0 1 2 3 4 5 6 7   
/// +-+-+-+-+-+-+-+-+
/// |class|  detail |
/// +-+-+-+-+-+-+-+-+
///         ^
///         |
///     1   |
/// 8 9 0 1 2 3 4 5 6    
/// +-+-+-+-+-+-+-+-+    
/// |      Code     |    
/// +-+-+-+-+-+-+-+-+    
/// ```
/// The meaning of the detail value is unknown until paired with a [`Class`](`crate::codec::Class`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Detail {
    value: u8,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    OutOfRange(u8),
}

impl Detail {
    /// Decode the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn decode(byte: u8) -> Self {
        Self { value: byte & MASK }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn encode(self) -> u8 {
        self.value()
    }

    // Construct a `Detail` from a u8.
    pub const fn from_value(value: u8) -> Result<Self, Error> {
        if !Self::value_valid(value) {
            return Err(Error::OutOfRange(value));
        }
        Ok(Self { value })
    }

    // Construct a `Detail` from a u8. Will panic if not valid value.
    pub const fn from_value_or_panic(value: u8) -> Self {
        match Self::from_value(value) {
            Ok(detail) => detail,
            Err(_) => panic!("Invalid value"),
        }
    }

    /// Get the numeric value of the detail.
    pub const fn value(&self) -> u8 {
        self.value
    }

    const fn value_valid(value: u8) -> bool {
        value <= MAX
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Detail, Error};

    #[rstest]
    fn decode_value_encode() {
        for value in 0..=0b000_11111 {
            let decoded = Detail::decode(value);
            let decoded_value = decoded.value();
            let encoded = decoded.encode();

            assert_eq!(Detail { value }, decoded);
            assert_eq!(value, decoded_value);
            assert_eq!(value, encoded);
        }
    }

    #[rstest]
    fn decoded_value_encode_zero() {
        for value in 0b000..=0b111 {
            let value = value << 5;

            let decoded = Detail::decode(value);
            let decoded_value = decoded.value();
            let encoded = decoded.encode();

            assert_eq!(Detail { value: 0 }, decoded);
            assert_eq!(0, decoded_value);
            assert_eq!(0, encoded);
        }
    }

    #[rstest]
    fn from_value() {
        for value in 0..=0b11111 {
            assert_eq!(Ok(Detail { value }), Detail::from_value(value))
        }
    }

    #[rstest]
    fn from_value_out_of_range() {
        for value in 0b1_00000..0b111_00000 {
            assert_eq!(Err(Error::OutOfRange(value)), Detail::from_value(value))
        }
    }
}
