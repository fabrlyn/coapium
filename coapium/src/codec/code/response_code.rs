use super::{Class, Code, Detail};

const CREATED: Detail = Detail::from_value_or_panic(1);
const DELETED: Detail = Detail::from_value_or_panic(2);
const VALID: Detail = Detail::from_value_or_panic(3);
const CHANGED: Detail = Detail::from_value_or_panic(4);
const CONTENT: Detail = Detail::from_value_or_panic(5);

const BAD_REQUEST: Detail = Detail::from_value_or_panic(0);
const UNAUTHORIZED: Detail = Detail::from_value_or_panic(1);
const BAD_OPTION: Detail = Detail::from_value_or_panic(2);
const FORBIDDEN: Detail = Detail::from_value_or_panic(3);
const NOT_FOUND: Detail = Detail::from_value_or_panic(4);
const METHOD_NOT_ALLOWED: Detail = Detail::from_value_or_panic(5);
const NOT_ACCEPTABLE: Detail = Detail::from_value_or_panic(6);
const PRECONDITION_FAILED: Detail = Detail::from_value_or_panic(12);
const REQUEST_ENTITY_TOO_LARGE: Detail = Detail::from_value_or_panic(13);
const UNSUPPORTED_CONTENT_FORMAT: Detail = Detail::from_value_or_panic(15);

const INTERNAL_SERVER_ERROR: Detail = Detail::from_value_or_panic(0);
const NOT_IMPLEMENTED: Detail = Detail::from_value_or_panic(1);
const BAD_GATEWAY: Detail = Detail::from_value_or_panic(2);
const SERVICE_UNAVAILABLE: Detail = Detail::from_value_or_panic(3);
const GATEWAY_TIMEOUT: Detail = Detail::from_value_or_panic(4);
const PROXYING_NOT_SUPPORTED: Detail = Detail::from_value_or_panic(5);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unassigned {
    value: Detail,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResponseCode {
    Success(Success),
    ClientError(ClientError),
    ServerError(ServerError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Success {
    Created,
    Deleted,
    Valid,
    Changed,
    Content,
    Unassigned(Unassigned),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClientError {
    BadRequest,
    Unauthorized,
    BadOption,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    PreconditionFailed,
    RequestEntityTooLarge,
    UnsupportedContentFormat,
    Unassigned(Unassigned),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ServerError {
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    ProxyingNotSupported,
    Unassigned(Unassigned),
}

impl ResponseCode {
    pub const fn encode(self) -> (Class, Detail) {
        match self {
            ResponseCode::Success(success) => (Class::Success, success.encode()),
            ResponseCode::ClientError(client_error) => (Class::ClientError, client_error.encode()),
            ResponseCode::ServerError(server_error) => (Class::ServerError, server_error.encode()),
        }
    }

    pub const fn decode_success(detail: Detail) -> ResponseCode {
        ResponseCode::Success(Success::decode(detail))
    }

    pub const fn decode_client_error(detail: Detail) -> ResponseCode {
        ResponseCode::ClientError(ClientError::decode(detail))
    }

    pub const fn decode_server_error(detail: Detail) -> ResponseCode {
        ResponseCode::ServerError(ServerError::decode(detail))
    }

    pub const fn is_success(&self) -> bool {
        match self {
            ResponseCode::Success(_) => true,
            _ => false,
        }
    }
}

impl Success {
    pub const fn decode(detail: Detail) -> Self {
        match detail {
            CREATED => Success::Created,
            DELETED => Success::Deleted,
            VALID => Success::Valid,
            CHANGED => Success::Changed,
            CONTENT => Success::Content,
            detail => Success::Unassigned(Unassigned { value: detail }),
        }
    }

    pub const fn encode(self) -> Detail {
        match self {
            Success::Created => CREATED,
            Success::Deleted => DELETED,
            Success::Valid => VALID,
            Success::Changed => CHANGED,
            Success::Content => CONTENT,
            Success::Unassigned(Unassigned { value }) => value,
        }
    }
}

impl ClientError {
    pub const fn decode(detail: Detail) -> Self {
        match detail {
            BAD_REQUEST => ClientError::BadRequest,
            UNAUTHORIZED => ClientError::Unauthorized,
            BAD_OPTION => ClientError::BadOption,
            FORBIDDEN => ClientError::Forbidden,
            NOT_FOUND => ClientError::NotFound,
            REQUEST_ENTITY_TOO_LARGE => ClientError::RequestEntityTooLarge,
            UNSUPPORTED_CONTENT_FORMAT => ClientError::UnsupportedContentFormat,
            detail => ClientError::Unassigned(Unassigned { value: detail }),
        }
    }

    pub const fn encode(self) -> Detail {
        match self {
            ClientError::BadRequest => BAD_REQUEST,
            ClientError::Unauthorized => UNAUTHORIZED,
            ClientError::BadOption => BAD_OPTION,
            ClientError::Forbidden => FORBIDDEN,
            ClientError::NotFound => NOT_FOUND,
            ClientError::MethodNotAllowed => METHOD_NOT_ALLOWED,
            ClientError::NotAcceptable => NOT_ACCEPTABLE,
            ClientError::PreconditionFailed => PRECONDITION_FAILED,
            ClientError::RequestEntityTooLarge => REQUEST_ENTITY_TOO_LARGE,
            ClientError::UnsupportedContentFormat => UNSUPPORTED_CONTENT_FORMAT,
            ClientError::Unassigned(Unassigned { value }) => value,
        }
    }
}

impl ServerError {
    pub const fn decode(detail: Detail) -> Self {
        match detail {
            INTERNAL_SERVER_ERROR => ServerError::InternalServerError,
            NOT_IMPLEMENTED => ServerError::NotImplemented,
            BAD_GATEWAY => ServerError::BadGateway,
            SERVICE_UNAVAILABLE => ServerError::ServiceUnavailable,
            GATEWAY_TIMEOUT => ServerError::GatewayTimeout,
            PROXYING_NOT_SUPPORTED => ServerError::ProxyingNotSupported,
            detail => ServerError::Unassigned(Unassigned { value: detail }),
        }
    }

    pub const fn encode(self) -> Detail {
        match self {
            ServerError::InternalServerError => INTERNAL_SERVER_ERROR,
            ServerError::NotImplemented => NOT_IMPLEMENTED,
            ServerError::BadGateway => BAD_GATEWAY,
            ServerError::ServiceUnavailable => SERVICE_UNAVAILABLE,
            ServerError::GatewayTimeout => GATEWAY_TIMEOUT,
            ServerError::ProxyingNotSupported => PROXYING_NOT_SUPPORTED,
            ServerError::Unassigned(Unassigned { value }) => value,
        }
    }
}

impl From<ResponseCode> for Code {
    fn from(value: ResponseCode) -> Self {
        Code::Response(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{
        Class, ClientError, Detail, ResponseCode, ServerError, Success, BAD_REQUEST, CREATED,
        INTERNAL_SERVER_ERROR,
    };

    #[rstest]
    #[case(ResponseCode::Success(Success::Created), (Class::Success, CREATED))]
    #[case(ResponseCode::ClientError(ClientError::BadRequest), (Class::ClientError, BAD_REQUEST))]
    #[case(ResponseCode::ServerError(ServerError::InternalServerError), (Class::ServerError, INTERNAL_SERVER_ERROR))]
    fn encode(#[case] response_code: ResponseCode, #[case] expected: (Class, Detail)) {
        assert_eq!(expected, response_code.encode())
    }
}
