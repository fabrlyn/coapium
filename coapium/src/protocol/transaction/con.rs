use std::time::Instant;

use crate::{
    codec::{MessageId, Token},
    protocol::{
        effect::{Effect, Effects},
        new_request::NewRequest,
        response,
        timeout::{ExchangeLifetimeTimeout, RetransmissionTimeout},
        transmission_parameters::ConfirmableParameters,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmableTransaction {
    pub acknowledged: bool,
    pub created_at: Instant,
    pub message_id: MessageId,
    pub request_data: Vec<u8>,
    pub retransmission_counter: u8,
    pub token: Token,
    pub transaction_parameters: ConfirmableParameters,
}

impl ConfirmableTransaction {
    pub fn new(
        message_id: MessageId,
        token: Token,
        request: NewRequest,
        parameters: ConfirmableParameters,
    ) -> Self {
        Self {
            acknowledged: false,
            created_at: Instant::now(),
            message_id,
            request_data: request.encode(message_id, token.clone()),
            retransmission_counter: 0,
            token,
            transaction_parameters: parameters,
        }
    }

    pub fn on_max_transmit_wait(&self) -> Result<Effects, Effects> {
        if self.acknowledged {
            return Ok(vec![]);
        }

        Err(vec![Effect::TransactionResolved(
            self.token.clone(),
            Err(response::Error::Timeout),
        )])
    }

    pub fn retransmit(
        &mut self,
        timeout: RetransmissionTimeout,
    ) -> Result<Vec<Effect>, Vec<Effect>> {
        if self.acknowledged {
            return Ok(vec![]);
        }

        if !self.can_retransmit() {
            return Err(vec![Effect::TransactionResolved(
                self.token.clone(),
                Err(response::Error::Timeout),
            )]);
        }

        self.retransmission_counter += 1;
        Ok(vec![
            timeout.next().into(),
            Effect::Transmit(self.request_data.clone()),
        ])
    }

    fn can_retransmit(&self) -> bool {
        self.retransmission_counter < self.transaction_parameters.max_retransmit()
    }

    pub fn acknowledged(&mut self) {
        self.acknowledged = true
    }

    pub fn initial_effects(&self) -> Effects {
        let retransmission_timeout =
            RetransmissionTimeout::new(self.message_id, &self.transaction_parameters);

        let exchange_lifetime_timeout =
            ExchangeLifetimeTimeout::new(self.message_id, &self.transaction_parameters);

        let transmit = Effect::Transmit(self.request_data.clone());

        vec![
            exchange_lifetime_timeout.into(),
            retransmission_timeout.into(),
            transmit,
        ]
    }
}

// TODO: Review these tests, muddy test cases
#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{
        codec::{message::GetOptions, Token},
        protocol::{
            effect::Effect,
            get::Get,
            reliability::Reliability,
            timeout::{ExchangeLifetimeTimeout, RetransmissionTimeout},
            transmission_parameters::{ConfirmableParameters, InitialRetransmissionFactor},
        },
        NewRequest,
    };

    use super::ConfirmableTransaction;

    #[rstest]
    fn initial_effects_with_minimum_initial_retransmission_timeout() {
        let confirmable_parameters =
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap());

        let transaction = ConfirmableTransaction::new(
            0.into(),
            Token::new().unwrap(),
            NewRequest::Get(Get {
                options: GetOptions::new(),
                reliability: Reliability::Confirmable(ConfirmableParameters::default(
                    InitialRetransmissionFactor::new(0.5).unwrap(),
                )),
            }),
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        );

        let effects = transaction.initial_effects();
        let expected_effects = vec![
            ExchangeLifetimeTimeout::new(transaction.message_id, &confirmable_parameters).into(),
            RetransmissionTimeout::new(transaction.message_id, &confirmable_parameters).into(),
            Effect::Transmit(transaction.request_data),
        ];
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn initial_effects_with_maximum_initial_retransmission_timeout() {
        let confirmable_parameters =
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap());

        let transaction = ConfirmableTransaction::new(
            0.into(),
            Token::new().unwrap(),
            NewRequest::Get(Get {
                options: GetOptions::new(),
                reliability: Reliability::Confirmable(ConfirmableParameters::default(
                    InitialRetransmissionFactor::new(0.5).unwrap(),
                )),
            }),
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        );

        let effects = transaction.initial_effects();
        let expected_effects = vec![
            ExchangeLifetimeTimeout::new(transaction.message_id, &confirmable_parameters).into(),
            RetransmissionTimeout::new(transaction.message_id, &confirmable_parameters).into(),
            Effect::Transmit(transaction.request_data),
        ];
        assert_eq!(expected_effects, effects);
    }
}
