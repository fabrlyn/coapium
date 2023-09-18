use crate::codec::{Code, MethodCode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialDurationFactor(f32);

impl TryFrom<f32> for InitialDurationFactor {
    type Error = ();

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 0.0 {
            return Err(());
        }

        if value > 1.0 {
            return Err(());
        }

        Ok(Self(value))
    }
}

impl InitialDurationFactor {
    pub fn value(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl From<Method> for Code {
    fn from(value: Method) -> Self {
        Code::Request(value.into())
    }
}

impl From<Method> for MethodCode {
    fn from(value: Method) -> Self {
        match value {
            Method::Get => MethodCode::Get,
            Method::Post => MethodCode::Post,
            Method::Put => MethodCode::Put,
            Method::Delete => MethodCode::Delete,
        }
    }
}
