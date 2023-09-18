use std::time::Duration;

use crate::codec::MessageId;

use super::transmission_parameters::{
    ConfirmableParameters, NonConfirmableParameters, ProbingRatePerSecond,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Factor {
    value: f32,
}

impl Factor {
    pub fn new(value: f32) -> Result<Self, ()> {
        if value < 0.0 {
            return Err(());
        }

        if value > 1.0 {
            return Err(());
        }

        Ok(Self { value })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetransmissionTimeout {
    timeout: Duration,
    message_id: MessageId,
}

impl RetransmissionTimeout {
    pub fn new(message_id: MessageId, confirmable_parameters: &ConfirmableParameters) -> Self {
        let variable_range =
            confirmable_parameters.max_ack_timeout() - confirmable_parameters.min_ack_timeout();

        let range = variable_range.mul_f32(confirmable_parameters.initial_retransmission_factor());

        let timeout = confirmable_parameters.min_ack_timeout() + range;

        Self {
            timeout,
            message_id,
        }
    }

    pub fn from_previous(previous_timeout: RetransmissionTimeout) -> Self {
        Self {
            timeout: previous_timeout.timeout * 2,
            ..previous_timeout
        }
    }

    pub fn next(self) -> Self {
        Self {
            timeout: self.timeout * 2,
            ..self
        }
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExchangeLifetimeTimeout {
    timeout: Duration,
    message_id: MessageId,
}

impl ExchangeLifetimeTimeout {
    pub fn new(message_id: MessageId, confirmable_parameters: &ConfirmableParameters) -> Self {
        Self {
            timeout: confirmable_parameters.exchange_lifetime(),
            message_id,
        }
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonRetransmissionTimeout {
    timeout: Duration,
    message_id: MessageId,
}

impl NonRetransmissionTimeout {
    pub fn new(
        message_id: &MessageId,
        data_len: usize,
        probing_rate_per_second: &ProbingRatePerSecond,
    ) -> Self {
        Self {
            timeout: Duration::from_secs_f32(probing_rate_per_second.value() * data_len as f32),
            message_id: *message_id,
        }
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonLifetimeTimeout {
    timeout: Duration,
    message_id: MessageId,
}

impl NonLifetimeTimeout {
    pub fn new(message_id: &MessageId, parameters: &NonConfirmableParameters) -> Self {
        Self {
            timeout: parameters.non_lifetime(),
            message_id: *message_id,
        }
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaxTransmitWaitTimeout {
    timeout: Duration,
    message_id: MessageId,
}

impl MaxTransmitWaitTimeout {
    pub fn new(message_id: &MessageId, confirmable_parameters: &ConfirmableParameters) -> Self {
        Self {
            timeout: confirmable_parameters.max_transmit_wait(),
            message_id: *message_id,
        }
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}
