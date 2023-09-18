use std::time::Duration;

use crate::protocol::timeout::{
    ExchangeLifetimeTimeout, MaxTransmitWaitTimeout, NonLifetimeTimeout, NonRetransmissionTimeout,
    RetransmissionTimeout,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Timeout {
    ExchangeLifetime(ExchangeLifetimeTimeout),
    MaxTransmitWait(MaxTransmitWaitTimeout),
    NonLifetime(NonLifetimeTimeout),
    NonRetransmission(NonRetransmissionTimeout),
    Retransmission(RetransmissionTimeout),
}

impl Timeout {
    pub fn duration(&self) -> &Duration {
        match self {
            Timeout::ExchangeLifetime(t) => t.timeout(),
            Timeout::MaxTransmitWait(t) => t.timeout(),
            Timeout::NonLifetime(t) => t.timeout(),
            Timeout::NonRetransmission(t) => t.timeout(),
            Timeout::Retransmission(t) => t.timeout(),
        }
    }
}

impl From<MaxTransmitWaitTimeout> for Timeout {
    fn from(value: MaxTransmitWaitTimeout) -> Self {
        Self::MaxTransmitWait(value)
    }
}

impl From<RetransmissionTimeout> for Timeout {
    fn from(value: RetransmissionTimeout) -> Self {
        Self::Retransmission(value)
    }
}

impl From<NonLifetimeTimeout> for Timeout {
    fn from(value: NonLifetimeTimeout) -> Self {
        Self::NonLifetime(value)
    }
}

impl From<NonRetransmissionTimeout> for Timeout {
    fn from(value: NonRetransmissionTimeout) -> Self {
        Self::NonRetransmission(value)
    }
}

impl From<ExchangeLifetimeTimeout> for Timeout {
    fn from(value: ExchangeLifetimeTimeout) -> Self {
        Self::ExchangeLifetime(value)
    }
}
