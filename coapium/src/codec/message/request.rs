use crate::codec::{Header, MethodCode};

use super::{get::Get, Error, Reliability};

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Get(Get),
    Post(()),
    Put(()),
    Delete(()),
}

impl Request {
    pub fn encode(self) -> Vec<u8> {
        match self {
            Request::Get(get) => get.encode(),
            Request::Post(_) => todo!(),
            Request::Put(_) => todo!(),
            Request::Delete(_) => todo!(),
        }
    }

    pub fn decode(
        _header: Header,
        _method_code: MethodCode,
        _reliability: Reliability,
        _remaining_bytes: &[u8],
    ) -> Result<Self, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::codec::{message::get_options::GetOptions, MessageId, Token};

    use super::{Get, Reliability, Request};

    #[rstest]
    #[case(
        Request::Get(
            Get::new(
                MessageId::from_value(3), 
                Reliability::Confirmable, 
                Token::from_value(vec![1, 2, 3]).unwrap(), 
                { 
                    let mut options = GetOptions::new();
                    options.set_uri_path("abc".try_into().unwrap());
                    options
                }
            )
        ),
        &[0b01_00_0011, 0b000_00001, 0, 3, 1, 2, 3, 0b1011_0011, 97, 98, 99]
    )]
    fn encode(#[case] request: Request, #[case] expected: &[u8]) {
        assert_eq!(expected, request.encode())
    }
}
