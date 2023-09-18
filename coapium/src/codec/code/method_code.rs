use super::Detail;

/// Numeric value of the GET method code
const GET: Detail = Detail::from_value_or_panic(1);

/// Numeric value of the POST method code
const POST: Detail = Detail::from_value_or_panic(2);

/// Numeric value of the PUT method code
const PUT: Detail = Detail::from_value_or_panic(3);

/// Numeric value of the DELETE method code
const DELETE: Detail = Detail::from_value_or_panic(4);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Unassigned {
    value: Detail,
}

/// The method code indicates that a message is a request along with the specific method and is parsed
/// from the [`Code`](`crate::codec::Code`) part of the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
///
/// ```markdown
/// 0                   1            
/// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Ver| T |  TKL  |      Code     |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
/// A method code is any [`Code`](`crate::codec::Code`) where the [`Class`](`crate::codec::code::Class`) value is [`RequestOrEmpty`](`crate::codec::code::Class::RequestOrEmpty`)
/// and the [`Detail`](`crate::codec::Detail`) is a non-zero value.
///
/// There are four method codes and they are denoated as:
/// - [`MethodCode::Get`](`MethodCode::Get`) / `0.01`.
/// - [`MethodCode::Post`](`MethodCode::Post`) / `0.02`.
/// - [`MethodCode::Put`](`MethodCode::Put`) / `0.03`.
/// - [`MethodCode::Delete`](`MethodCode::Delete`) / `0.04`.
///
/// All other values, except `0.00`, are considered [`Unassigned`](`crate::codec::MethodCode::Unassigned`).
///
/// The numeric value of each method code, including unassigned method codes, only represent the [`Detail`](`crate::codec::code::Detail`)
/// since the class is assumed to be [`RequestOrEmpty`](`crate::codec::code::Class::RequestOrEmpty`).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MethodCode {
    /// Present in a GET-request message.
    /// Value defined by [`GET`](`GET`).
    Get,

    /// Present in a POST-request message.
    /// Value defined by [`POST`](`POST`).
    Post,

    /// Present in a PUT-request message.
    /// Value defined by [`PUT`](`PUT`).
    Put,

    /// Present in a DELETE-request message.
    /// Value defined by [`DELETE`](`DELETE`).
    Delete,

    /// All other [`Detail`](`crate::codec::code::Detail`) values in [`Code`](`crate::codec::Code`) which is not yet assigned or unsupported.
    Unassigned(Unassigned),
}

impl MethodCode {
    // TODO: This should be `From<CodeDetails> for MethodCode`
    /// Decode the byte from the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn decode(detail: Detail) -> Self {
        match detail {
            GET => Self::Get,
            POST => Self::Post,
            PUT => Self::Put,
            DELETE => Self::Delete,
            detail => Self::Unassigned(Unassigned { value: detail }),
        }
    }

    /// Encode to a byte formatted to fit into the [message header](https://datatracker.ietf.org/doc/html/rfc7252#section-3).
    pub const fn encode(self) -> Detail {
        match self {
            Self::Get => GET,
            Self::Post => POST,
            Self::Put => PUT,
            Self::Delete => DELETE,
            Self::Unassigned(Unassigned { value }) => value,
        }
    }

    /// Returns `true` if method code is [`Get`](`MethodCode::Get`)
    pub const fn is_get(&self) -> bool {
        match self {
            Self::Get => true,
            _ => false,
        }
    }

    /// Returns `true` if method code is [`Post`](`MethodCode::Post`)
    pub const fn is_post(&self) -> bool {
        match self {
            Self::Post => true,
            _ => false,
        }
    }

    /// Returns `true` if method code is [`Put`](`MethodCode::Put`)
    pub const fn is_put(&self) -> bool {
        match self {
            Self::Put => true,
            _ => false,
        }
    }

    /// Returns `true` if method code is [`Delete`](`MethodCode::Delete`)
    pub const fn is_delete(&self) -> bool {
        match self {
            Self::Delete => true,
            _ => false,
        }
    }

    /// Returns `true` if method code is [`Unassigned`](`MethodCode::Unassigned`)
    pub const fn is_unassigned(&self) -> bool {
        match self {
            Self::Unassigned(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Detail, MethodCode, Unassigned, DELETE, GET, POST, PUT};

    #[rstest]
    #[case(GET, MethodCode::Get)]
    #[case(POST, MethodCode::Post)]
    #[case(PUT, MethodCode::Put)]
    #[case(DELETE, MethodCode::Delete)]
    #[case(Detail::from_value(5).unwrap(), MethodCode::Unassigned(Unassigned{value: Detail::from_value(5).unwrap()}))]
    fn decode(#[case] detail: Detail, #[case] expected: MethodCode) {
        assert_eq!(expected, MethodCode::decode(detail))
    }

    #[rstest]
    #[case(MethodCode::Get, GET)]
    #[case(MethodCode::Post, POST)]
    #[case(MethodCode::Put, PUT)]
    #[case(MethodCode::Delete, DELETE)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), Detail::from_value_or_panic(5))]
    fn encode(#[case] method_code: MethodCode, #[case] expected: Detail) {
        assert_eq!(expected, method_code.encode())
    }

    #[rstest]
    #[case(MethodCode::Get, true)]
    #[case(MethodCode::Post, false)]
    #[case(MethodCode::Put, false)]
    #[case(MethodCode::Delete, false)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), false)]
    fn is_get(#[case] method_code: MethodCode, #[case] expected: bool) {
        assert_eq!(expected, method_code.is_get())
    }

    #[rstest]
    #[case(MethodCode::Get, false)]
    #[case(MethodCode::Post, true)]
    #[case(MethodCode::Put, false)]
    #[case(MethodCode::Delete, false)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), false)]
    fn is_post(#[case] method_code: MethodCode, #[case] expected: bool) {
        assert_eq!(expected, method_code.is_post())
    }

    #[rstest]
    #[case(MethodCode::Get, false)]
    #[case(MethodCode::Post, false)]
    #[case(MethodCode::Put, true)]
    #[case(MethodCode::Delete, false)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), false)]
    fn is_put(#[case] method_code: MethodCode, #[case] expected: bool) {
        assert_eq!(expected, method_code.is_put())
    }

    #[rstest]
    #[case(MethodCode::Get, false)]
    #[case(MethodCode::Post, false)]
    #[case(MethodCode::Put, false)]
    #[case(MethodCode::Delete, true)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), false)]
    fn is_delete(#[case] method_code: MethodCode, #[case] expected: bool) {
        assert_eq!(expected, method_code.is_delete())
    }

    #[rstest]
    #[case(MethodCode::Get, false)]
    #[case(MethodCode::Post, false)]
    #[case(MethodCode::Put, false)]
    #[case(MethodCode::Delete, false)]
    #[case(MethodCode::Unassigned(Unassigned{value: Detail::from_value_or_panic(5)}), true)]
    fn is_unassigned(#[case] method_code: MethodCode, #[case] expected: bool) {
        assert_eq!(expected, method_code.is_unassigned())
    }
}
