use Class::*;

/// Steps to shift to encode/decode a byte
const SHIFT: u8 = 5;

/// Numeric value of request or empty
const REQUEST_OR_EMPTY: u8 = 0;

/// Numeric value of success
const SUCCESS: u8 = 2;

/// Numeric value of client error
const CLIENT_ERROR: u8 = 4;

/// Numeric value of server error
const SERVER_ERROR: u8 = 5;

/// The class value of the [`Code`](`crate::codec::Code`) in a [`Message`](`crate::codec::Message`).
///
/// The class(`class`) consists of a 3-bit value and are the first bits in the [`Code`](`crate::codec::Code`)
/// field in the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
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
///
/// A class has four defined variants:
/// - [`RequestOrEmpty`](`Class::RequestOrEmpty`)
/// - [`Success`](`Class::Success`)
/// - [`ClientError`](`Class::ClientError`)
/// - [`ServerError`](`Class::ServerError`)
///
/// Other possible values are allowed but are reserved and will be decoded as [`Reserved`](`Class::Reserved`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    /// Value(`0`) defined by [`REQUEST_OR_EMPTY`](`REQUEST_OR_EMPTY`).
    ///
    /// A [`request code`](`crate::codec::Code::Request`) and an [`empty code`](`crate::codec::Code::Empty`)
    /// share the same class value which means that
    /// a class value alone can not decide if a code is a request or empty.
    ///
    /// An empty code is a code where `class` and `detail` are both `0`, also denoted as `0.00`.
    ///
    /// All other `detail` values in combination with a zero(`0`) class value is are request codes.
    RequestOrEmpty,

    /// Value(`2`) defined by [`SUCCESS`](`SUCCESS`).
    Success,

    /// Value(`4`) defined by [`CLIENT_ERROR`](`CLIENT_ERROR`).
    ClientError,

    /// Value(`5`) defined by [`SERVER_ERROR`](`SERVER_ERROR`).
    ServerError,

    /// The reserved value is stored in `value` field.
    Reserved { value: u8 },
}

impl Class {
    /// Decode the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn decode(byte: u8) -> Self {
        match byte >> SHIFT {
            REQUEST_OR_EMPTY => RequestOrEmpty,
            SUCCESS => Success,
            CLIENT_ERROR => ClientError,
            SERVER_ERROR => ServerError,
            value => Reserved { value },
        }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn encode(self) -> u8 {
        self.value() << SHIFT
    }

    /// Returns `true` if the class  is [`ClientError`](`Class::ClientError`)
    pub const fn is_client_error(&self) -> bool {
        match self {
            ClientError => true,
            _ => false,
        }
    }

    /// Returns `true` if the class  is [`Success`](`Class::Success`)
    pub const fn is_success(&self) -> bool {
        match self {
            Success => true,
            _ => false,
        }
    }

    /// Returns `true` if the class  is [`RequestOrEmpty`](`Class::RequestOrEmpty`)
    pub const fn is_request_or_empty(&self) -> bool {
        match self {
            RequestOrEmpty => true,
            _ => false,
        }
    }

    /// Returns `true` if the class  is [`Reserved`](`Class::Reserved`)
    pub const fn is_reserved(&self) -> bool {
        match self {
            Reserved { .. } => true,
            _ => false,
        }
    }

    /// Returns `true` if the class  is [`ServerError`](`Class::ServerError`)
    pub const fn is_server_error(&self) -> bool {
        match self {
            ServerError => true,
            _ => false,
        }
    }

    /// Get the numeric value of the class.
    ///
    /// Possible return values:
    /// - [`REQUEST_OR_EMPTY`](`REQUEST_OR_EMPTY`)
    /// - [`SUCCESS`](`SUCCESS`)
    /// - [`CLIENT_ERROR`](`CLIENT_ERROR`)
    /// - [`SERVER_ERROR`](`SERVER_ERROR`)
    /// - Reserved, any value outside of the defined variants.
    pub const fn value(&self) -> u8 {
        match self {
            RequestOrEmpty => REQUEST_OR_EMPTY,
            Success => SUCCESS,
            ClientError => CLIENT_ERROR,
            ServerError => SERVER_ERROR,
            Reserved { value } => *value,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use std::ops::RangeInclusive;

    use super::{
        Class::{self, *},
        CLIENT_ERROR, REQUEST_OR_EMPTY, SERVER_ERROR, SUCCESS,
    };

    #[rstest]
    #[case(0b000_00000..=0b000_11111, RequestOrEmpty)]
    #[case(0b010_00000..=0b010_11111, Success)]
    #[case(0b100_00000..=0b100_11111, ClientError)]
    #[case(0b101_00000..=0b101_11111, ServerError)]
    fn decode(#[case] input_range: RangeInclusive<u8>, #[case] expected: Class) {
        for input in input_range {
            assert_eq!(expected, Class::decode(input));
        }
    }

    #[rstest]
    #[case(0b011_00000..=0b011_11111)]
    #[case(0b001_00000..=0b001_11111)]
    #[case(0b110_00000..=0b110_11111)]
    #[case(0b111_00000..=0b111_11111)]
    fn decode_reserved(#[case] input_range: RangeInclusive<u8>) {
        for input in input_range {
            match Class::decode(input) {
                Reserved { .. } => {}
                _ => panic!(),
            }
        }
    }

    #[rstest]
    #[case(RequestOrEmpty, 0b000_00000)]
    #[case(Success, 0b010_00000)]
    #[case(ClientError, 0b100_00000)]
    #[case(ServerError, 0b101_00000)]
    #[case(Reserved { value: 6 }, 0b110_00000)]
    fn encode(#[case] class: Class, #[case] expected: u8) {
        assert_eq!(expected, class.encode())
    }

    #[rstest]
    #[case(RequestOrEmpty, false)]
    #[case(Success, false)]
    #[case(ClientError, true)]
    #[case(ServerError, false)]
    #[case(Reserved { value: 7 }, false)]
    fn is_client_error(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_client_error());
    }

    #[rstest]
    #[case(RequestOrEmpty, false)]
    #[case(Success, true)]
    #[case(ClientError, false)]
    #[case(ServerError, false)]
    #[case(Reserved { value: 7 }, false)]
    fn is_success(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_success());
    }

    #[rstest]
    #[case(RequestOrEmpty, true)]
    #[case(Success, false)]
    #[case(ClientError, false)]
    #[case(ServerError, false)]
    #[case(Reserved { value: 7 }, false)]
    fn is_request_or_empty(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_request_or_empty());
    }

    #[rstest]
    #[case(RequestOrEmpty, false)]
    #[case(Success, false)]
    #[case(ClientError, false)]
    #[case(ServerError, false)]
    #[case(Reserved { value: 7 }, true)]
    fn is_reserved(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_reserved());
    }

    #[rstest]
    #[case(RequestOrEmpty, false)]
    #[case(Success, false)]
    #[case(ClientError, false)]
    #[case(ServerError, true)]
    #[case(Reserved { value: 7 }, false)]
    fn is_server_error(#[case] class: Class, #[case] expected: bool) {
        assert_eq!(expected, class.is_server_error());
    }

    #[rstest]
    #[case(RequestOrEmpty, REQUEST_OR_EMPTY)]
    #[case(Success, SUCCESS)]
    #[case(ClientError, CLIENT_ERROR)]
    #[case(ServerError, SERVER_ERROR)]
    #[case(Reserved { value: 7 }, 7)]
    fn value(#[case] class: Class, #[case] expected: u8) {
        assert_eq!(expected, class.value());
    }
}
