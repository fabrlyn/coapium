use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransmissionParamters {
    Confirmable(ConfirmableParameters),
    NonConfirmable(NonConfirmableParameters),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AckRandomFactor {
    value: f32,
}

impl AckRandomFactor {
    pub fn new(value: f32) -> Result<Self, ()> {
        if value < 1.0 {
            return Err(());
        }

        Ok(Self { value })
    }
}

impl Default for AckRandomFactor {
    fn default() -> Self {
        Self::new(1.5).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AckTimeout {
    value: Duration,
}

impl AckTimeout {
    pub fn new(value: Duration) -> Result<Self, ()> {
        if value < Duration::from_secs(1) {
            return Err(());
        }

        Ok(Self { value })
    }
}

impl Default for AckTimeout {
    fn default() -> Self {
        Self::new(Duration::from_secs(2)).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaxRetransmit {
    value: u8,
}

impl MaxRetransmit {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}

impl Default for MaxRetransmit {
    fn default() -> Self {
        Self::new(4)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfirmableParameters {
    ack_timeout: AckTimeout,
    ack_random_factor: AckRandomFactor,
    initial_retransmission_factor: InitialRetransmissionFactor,
    max_retransmit: MaxRetransmit,
}

impl ConfirmableParameters {
    pub fn new(
        ack_timeout: AckTimeout,
        ack_random_factor: AckRandomFactor,
        initial_retransmission_factor: InitialRetransmissionFactor,
        max_retransmit: MaxRetransmit,
    ) -> Self {
        Self {
            ack_timeout,
            ack_random_factor,
            initial_retransmission_factor,
            max_retransmit,
        }
    }

    pub fn default(initial_retransmission_factor: InitialRetransmissionFactor) -> Self {
        Self {
            ack_timeout: AckTimeout::new(Duration::from_secs(2)).unwrap(),
            ack_random_factor: AckRandomFactor::new(1.5).unwrap(),
            initial_retransmission_factor,
            max_retransmit: Default::default(),
        }
    }

    pub fn max_transmit_wait(&self) -> Duration {
        self.ack_timeout().mul_f32(self.ack_random_factor())
            * ((self.max_retransmit() + 1).pow(2) - 1).into()
    }

    pub fn ack_timeout(&self) -> Duration {
        self.ack_timeout.value
    }

    pub fn min_ack_timeout(&self) -> Duration {
        self.ack_timeout.value
    }

    pub fn max_ack_timeout(&self) -> Duration {
        self.ack_timeout.value.mul_f32(self.ack_random_factor.value)
    }

    pub fn ack_random_factor(&self) -> f32 {
        self.ack_random_factor.value
    }

    pub fn initial_retransmission_factor(&self) -> f32 {
        self.initial_retransmission_factor.value
    }

    pub fn max_retransmit(&self) -> u8 {
        self.max_retransmit.value
    }

    /*
        Note that there is no need to consider
        MAX_TRANSMIT_WAIT if the configuration is chosen such that the
        last waiting period (ACK_TIMEOUT * (2 ** MAX_RETRANSMIT) or the
        difference between MAX_TRANSMIT_SPAN and MAX_TRANSMIT_WAIT) is
        less than MAX_LATENCY -- which is a likely choice, as MAX_LATENCY
        is a worst-case value unlikely to be met in the real world.  In
        this case, EXCHANGE_LIFETIME simplifies to:
        MAX_TRANSMIT_SPAN + (2 * MAX_LATENCY) + PROCESSING_DELAY
    */
    pub fn exchange_lifetime(&self) -> Duration {
        self.max_transmit_span() + (2 * self.max_latency()) + self.processing_delay()
    }

    pub fn max_transmit_span(&self) -> Duration {
        self.ack_timeout().mul_f32(self.ack_random_factor())
            * (self.max_retransmit().pow(2) - 1).into()
    }

    pub fn max_latency(&self) -> Duration {
        Duration::from_secs(100)
    }

    pub fn processing_delay(&self) -> Duration {
        self.ack_timeout()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialRetransmissionFactor {
    value: f32,
}

impl InitialRetransmissionFactor {
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
pub struct ProbingRatePerSecond {
    value: f32,
}

impl ProbingRatePerSecond {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl Default for ProbingRatePerSecond {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl From<f32> for ProbingRatePerSecond {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonConfirmableParameters {
    probing_rate_per_second: Option<ProbingRatePerSecond>,
    ack_timeout: AckTimeout,
    ack_random_factor: AckRandomFactor,
    max_retransmit: MaxRetransmit,
}

impl NonConfirmableParameters {
    pub fn default() -> Self {
        Self {
            ack_timeout: AckTimeout::default(),
            ack_random_factor: AckRandomFactor::default(),
            max_retransmit: MaxRetransmit::default(),
            probing_rate_per_second: None,
        }
    }

    pub fn new(
        ack_timeout: AckTimeout,
        ack_random_factor: AckRandomFactor,
        max_retransmit: MaxRetransmit,
        probing_rate_per_second: Option<ProbingRatePerSecond>,
    ) -> Self {
        Self {
            probing_rate_per_second,
            ack_timeout,
            ack_random_factor,
            max_retransmit,
        }
    }

    pub fn probing_rate_per_second(&self) -> &Option<ProbingRatePerSecond> {
        &self.probing_rate_per_second
    }

    pub fn non_lifetime(&self) -> Duration {
        self.max_transmit_span() + self.max_latency()
    }

    pub fn max_transmit_span(&self) -> Duration {
        self.ack_timeout().mul_f32(self.ack_random_factor())
            * (self.max_retransmit().pow(2) - 1).into()
    }

    fn max_latency(&self) -> Duration {
        Duration::from_secs(100)
    }

    pub fn ack_timeout(&self) -> Duration {
        self.ack_timeout.value
    }

    pub fn min_ack_timeout(&self) -> Duration {
        self.ack_timeout.value
    }

    pub fn max_ack_timeout(&self) -> Duration {
        self.ack_timeout.value.mul_f32(self.ack_random_factor.value)
    }

    pub fn ack_random_factor(&self) -> f32 {
        self.ack_random_factor.value
    }

    pub fn max_retransmit(&self) -> u8 {
        self.max_retransmit.value
    }
}
