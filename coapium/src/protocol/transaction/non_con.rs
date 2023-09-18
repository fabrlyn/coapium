use crate::protocol::{
    new_request::NewRequest,
    timeout::{NonLifetimeTimeout, NonRetransmissionTimeout},
    transmission_parameters::NonConfirmableParameters,
};
use std::time::Instant;

use crate::{
    codec::{MessageId, Token},
    protocol::effect::{Effect, Effects},
};

#[derive(Debug)]
pub struct NonConfirmableTransacation {
    pub created_at: Instant,
    pub token: Token,
    pub message_id: MessageId,
    pub request_data: Vec<u8>,
    pub transaction_parameters: NonConfirmableParameters,
}

impl NonConfirmableTransacation {
    pub fn new(
        message_id: MessageId,
        token: Token,
        request: NewRequest,
        transaction_parameters: NonConfirmableParameters,
    ) -> Self {
        Self {
            created_at: Instant::now(),
            message_id,
            request_data: request.encode(message_id, token.clone()),
            token,
            transaction_parameters,
        }
    }

    pub fn retransmit(&mut self) -> Result<Vec<Effect>, Vec<Effect>> {
        if let Some(timeout) = self.timeout() {
            Ok(vec![timeout.into()])
        } else {
            Ok(vec![])
        }
    }

    pub fn initial_effects(&self) -> Effects {
        let mut effects = vec![];

        effects
            .push(NonLifetimeTimeout::new(&self.message_id, &self.transaction_parameters).into());

        if let Some(timeout) = self.timeout() {
            effects.push(timeout.into());
        }

        effects.push(Effect::Transmit(self.request_data.clone()));

        effects
    }

    fn timeout(&self) -> Option<NonRetransmissionTimeout> {
        if let Some(probing_rate_per_second) = self.transaction_parameters.probing_rate_per_second()
        {
            Some(NonRetransmissionTimeout::new(
                &self.message_id,
                self.request_data.len(),
                probing_rate_per_second,
            ))
        } else {
            None
        }
    }
}
