pub mod con;
pub mod non_con;

use std::time::{Duration, Instant};

use crate::codec::{MessageId, Token};

use self::{con::ConfirmableTransaction, non_con::NonConfirmableTransacation};

use super::{
    effect::{Effect, Effects},
    new_request::NewRequest,
    reliability::Reliability,
    response,
};

pub const ACK_RANDOM_FACTOR: f32 = 1.5;
pub const ACK_TIMEOUT: Duration = Duration::from_secs(2);
pub const DEFAULT_LEISURE: Duration = Duration::from_secs(5);
pub const EXCHANGE_LIFETIME: Duration = Duration::from_secs(247);
pub const MAX_LATENCY: Duration = Duration::from_secs(100);
pub const MAX_RETRANSMIT: u8 = 4;
pub const MAX_RTT: Duration = Duration::from_secs(202);
pub const MAX_TRANSMIT: u8 = 4;
pub const MAX_TRANSMIT_SPAN: Duration = Duration::from_secs(45);
pub const MAX_TRANSMIT_WAIT: Duration = Duration::from_secs(93);
pub const NON_LIFETIME: Duration = Duration::from_secs(145);
pub const NSTART: usize = 1;
pub const PATH_MTU: usize = 1152;
pub const PROBING_RATE_PER_SECOND: u8 = 1;
pub const PROCESSING_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub enum Transaction {
    Confirmable(ConfirmableTransaction),
    NonConfirmable(NonConfirmableTransacation),
}

impl Transaction {
    pub fn new(message_id: MessageId, token: Token, request: NewRequest) -> Self {
        match request.reliability() {
            Reliability::Confirmable(parameters) => Transaction::Confirmable(
                ConfirmableTransaction::new(message_id, token, request, parameters),
            ),
            Reliability::NonConfirmable(parameters) => Transaction::NonConfirmable(
                NonConfirmableTransacation::new(message_id, token, request, parameters),
            ),
        }
    }

    pub fn increment_retransmit_counter(&mut self) {
        match self {
            Transaction::Confirmable(t) => t.retransmission_counter += 1,
            Transaction::NonConfirmable(_t) => {}
        }
    }

    pub fn created_at(&self) -> Instant {
        match self {
            Transaction::Confirmable(t) => t.created_at,
            Transaction::NonConfirmable(t) => t.created_at,
        }
    }

    pub fn request_data(&self) -> &[u8] {
        match self {
            Transaction::Confirmable(t) => &t.request_data,
            Transaction::NonConfirmable(t) => &t.request_data,
        }
    }

    pub fn is_acknowledged(&self) -> bool {
        match self {
            Transaction::Confirmable(t) => t.acknowledged,
            Transaction::NonConfirmable(_) => false,
        }
    }
    pub fn is_non_confirmable(&self) -> bool {
        match self {
            Transaction::NonConfirmable(_) => true,
            _ => false,
        }
    }

    pub fn message_id(&self) -> MessageId {
        match self {
            Transaction::Confirmable(t) => t.message_id,
            Transaction::NonConfirmable(t) => t.message_id,
        }
    }
    pub fn retransmit_counter(&self) -> u8 {
        match self {
            Transaction::Confirmable(t) => t.retransmission_counter,
            Transaction::NonConfirmable(_t) => 0,
        }
    }

    pub fn token(&self) -> &Token {
        match self {
            Transaction::Confirmable(t) => &t.token,
            Transaction::NonConfirmable(t) => &t.token,
        }
    }

    pub fn timeout(self) -> Effect {
        let token = match self {
            Transaction::Confirmable(transcation) => transcation.token,
            Transaction::NonConfirmable(transaction) => transaction.token,
        };

        Effect::TransactionResolved(token, Err(response::Error::Timeout))
    }

    pub fn acknowledged(&mut self) {
        match self {
            Self::Confirmable(transcation) => transcation.acknowledged(),
            _ => {}
        }
    }

    pub fn initial_effects(&self) -> Effects {
        match self {
            Self::Confirmable(transaction) => transaction.initial_effects(),
            Self::NonConfirmable(transaction) => transaction.initial_effects(),
        }
    }
}
