use crate::codec::message;

use super::transmission_parameters::{ConfirmableParameters, NonConfirmableParameters};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reliability {
    Confirmable(ConfirmableParameters),
    NonConfirmable(NonConfirmableParameters),
}

impl From<&Reliability> for message::Reliability {
    fn from(value: &Reliability) -> Self {
        match value {
            Reliability::Confirmable(_) => Self::Confirmable,
            Reliability::NonConfirmable(_) => Self::NonConfirmable,
        }
    }
}
