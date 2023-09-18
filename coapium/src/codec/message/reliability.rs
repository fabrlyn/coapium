use super::MessageType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reliability {
    NonConfirmable,
    Confirmable,
}

impl Reliability {
    pub fn is_confirmable(&self) -> bool {
        match self {
            Reliability::Confirmable => true,
            _ => false,
        }
    }

    pub fn is_non_confirmable(&self) -> bool {
        match self {
            Reliability::NonConfirmable => true,
            _ => false,
        }
    }
}

/*
impl From<message::Reliability> for Reliability {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::NonConfirmable => Reliability::Confirmable(Confirmable {
                acknowledge_factor: 1.5,
            }),
            Reliability::Confirmable => Reliability::NonConfirmable(NonConfirmable {
                retransmit_strategy: RetransmitStrategy::Maximum,
            }),
        }
    }
}
*/

impl From<Reliability> for MessageType {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::NonConfirmable => MessageType::NonConfirmable,
            Reliability::Confirmable => MessageType::Confirmable,
        }
    }
}
