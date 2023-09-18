pub mod class;
pub mod detail;
pub mod method_code;
pub mod reserved_code;
pub mod response_code;

pub use class::Class;
pub use detail::Detail;
pub use method_code::MethodCode;

use self::{reserved_code::ReservedCode, response_code::ResponseCode};

const DETAIL_ZERO: Detail = Detail::from_value_or_panic(0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Code {
    Empty,
    Request(MethodCode),
    Response(ResponseCode),
    Reserved(ReservedCode),
}

impl Code {
    pub const fn decode(byte: u8) -> Self {
        let class = Class::decode(byte);
        let detail = Detail::decode(byte);

        match (class, detail) {
            (Class::RequestOrEmpty, DETAIL_ZERO) => Code::Empty,
            (Class::RequestOrEmpty, detail) => Code::Request(MethodCode::decode(detail)),
            (Class::Success, detail) => Code::Response(ResponseCode::decode_success(detail)),
            (Class::ClientError, detail) => {
                Code::Response(ResponseCode::decode_client_error(detail))
            }
            (Class::ServerError, detail) => {
                Code::Response(ResponseCode::decode_server_error(detail))
            }
            (Class::Reserved { .. }, detail) => Code::Reserved(ReservedCode::new(class, detail)),
        }
    }

    pub const fn encode(self) -> u8 {
        let (class, detail) = match self {
            Code::Empty => (Class::RequestOrEmpty, DETAIL_ZERO),
            Code::Request(method) => (Class::RequestOrEmpty, method.encode()),
            Code::Response(response) => response.encode(),
            Code::Reserved(reserved) => reserved.encode(),
        };

        class.encode() | detail.encode()
    }

    pub const fn is_empty(&self) -> bool {
        match self {
            Code::Empty => true,
            _ => false,
        }
    }

    pub const fn is_request(&self) -> bool {
        match self {
            Code::Request(_) => true,
            _ => false,
        }
    }

    pub const fn is_response(&self) -> bool {
        match self {
            Code::Response(_) => true,
            _ => false,
        }
    }

    pub const fn is_reserved(&self) -> bool {
        match self {
            Code::Reserved { .. } => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        response_code::{ClientError, ServerError, Success},
        Class, Code, Detail, MethodCode, ReservedCode, ResponseCode,
    };

    #[rstest]
    #[case(0b000_00000, Code::Empty)]
    #[case(0b000_00001, Code::Request(MethodCode::Get))]
    #[case(0b001_00001, Code::Reserved(ReservedCode::new(Class::Reserved{value: 1}, Detail::from_value_or_panic(1))))]
    #[case(0b010_00001, Code::Response(ResponseCode::Success(Success::Created)))]
    #[case(0b011_00001, Code::Reserved(ReservedCode::new(Class::Reserved{value: 3}, Detail::from_value_or_panic(1))))]
    #[case(
        0b100_00001,
        Code::Response(ResponseCode::ClientError(ClientError::Unauthorized))
    )]
    #[case(
        0b101_00001,
        Code::Response(ResponseCode::ServerError(ServerError::NotImplemented))
    )]
    #[case(0b110_00001, Code::Reserved(ReservedCode::new(Class::Reserved{value: 6}, Detail::from_value_or_panic(1))))]
    #[case(0b111_00001, Code::Reserved(ReservedCode::new(Class::Reserved{value: 7}, Detail::from_value_or_panic(1))))]
    fn decode_encode(#[case] byte: u8, #[case] expected: Code) {
        let code = Code::decode(byte);

        assert_eq!(expected, code);
        assert_eq!(code.encode(), byte);
    }

    #[rstest]
    #[case(Code::Empty, true)]
    #[case(Code::Request(MethodCode::Get), false)]
    #[case(Code::Response(ResponseCode::Success(Success::Content)), false)]
    #[case(Code::Reserved(ReservedCode::new(Class::Reserved { value: 7 }, Detail::from_value_or_panic(1))), false)]
    fn is_empty(#[case] code: Code, #[case] expected: bool) {
        assert_eq!(expected, code.is_empty())
    }

    #[rstest]
    #[case(Code::Empty, false)]
    #[case(Code::Request(MethodCode::Get), true)]
    #[case(Code::Response(ResponseCode::Success(Success::Content)), false)]
    #[case(Code::Reserved(ReservedCode::new(Class::Reserved { value: 7 }, Detail::from_value_or_panic(1))), false)]
    fn is_request(#[case] code: Code, #[case] expected: bool) {
        assert_eq!(expected, code.is_request())
    }

    #[rstest]
    #[case(Code::Empty, false)]
    #[case(Code::Request(MethodCode::Get), false)]
    #[case(Code::Response(ResponseCode::Success(Success::Content)), true)]
    #[case(Code::Reserved(ReservedCode::new(Class::Reserved { value: 7 }, Detail::from_value_or_panic(1))), false)]
    fn is_response(#[case] code: Code, #[case] expected: bool) {
        assert_eq!(expected, code.is_response())
    }

    #[rstest]
    #[case(Code::Empty, false)]
    #[case(Code::Request(MethodCode::Get), false)]
    #[case(Code::Response(ResponseCode::Success(Success::Content)), false)]
    #[case(Code::Reserved(ReservedCode::new(Class::Reserved { value: 7 }, Detail::from_value_or_panic(1))), true)]
    fn is_reserved(#[case] code: Code, #[case] expected: bool) {
        assert_eq!(expected, code.is_reserved())
    }
}
