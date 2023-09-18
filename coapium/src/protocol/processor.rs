use std::collections::VecDeque;

use crate::codec::{
    self, message::Message, message_id::MessageId, token::Token, Acknowledgement, Piggyback, Reset,
};

use super::{
    effect::{Effect, Effects, Timeout},
    event::Event,
    message_id_store::MessageIdStore,
    new_request::NewRequest,
    response,
    timeout::{
        ExchangeLifetimeTimeout, MaxTransmitWaitTimeout, NonLifetimeTimeout, RetransmissionTimeout,
    },
    transaction::Transaction,
    transaction_store::TransactionStore,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Other(String),
}

impl Error {
    pub fn other<S: ToString>(message: S) -> Self {
        Self::Other(message.to_string())
    }
}

pub type Result = std::result::Result<Effects, Error>;

#[derive(Debug)]
pub struct Processor {
    queued: VecDeque<(NewRequest, Token)>,
    transaction_store: TransactionStore,
    message_id_store: MessageIdStore,
}

impl Processor {
    pub fn new(message_id_store: MessageIdStore) -> Self {
        Self {
            queued: Default::default(),
            transaction_store: Default::default(),
            message_id_store,
        }
    }

    pub fn tick(&mut self, event: Event) -> Result {
        match event {
            Event::TransactionRequested(request, token) => {
                self.on_transaction_requested(request, token)
            }
            Event::TransactionCanceled(_) => Ok(vec![]),
            Event::TimeoutReached(timeout) => self.on_timeout_reached(timeout),
            Event::DataReceived(data) => self.on_data_received(data),
        }
    }

    fn at_capacity(&self) -> bool {
        return self.transaction_store.at_max_inflight_capacity()
            || self.message_id_store.at_capacity();
    }

    fn claim_message_id(&mut self) -> std::result::Result<MessageId, Error> {
        let Some(message_id) = self.message_id_store.claim() else {
            return Err(Error::other("Failed to claim message id"));
        };

        Ok(message_id)
    }

    fn on_data_received(&mut self, data: Vec<u8>) -> Result {
        let message = Message::decode(&data)
            .map_err(|e| Error::other(format!("Failed to parse message => {e:?}")))?;

        match message {
            Message::Acknowledgement(acknowledgement) => self.on_acknowledgement(acknowledgement),
            Message::Piggyback(piggyback) => self.on_piggyback(piggyback),
            Message::Request(_) => Ok(vec![]),
            Message::Reset(reset) => self.on_reset(reset),
            Message::Response(response) => self.on_response(response),
            Message::Reserved(_) => Ok(vec![]),
        }
    }

    fn dequeue_request(&mut self) -> Result {
        if self.at_capacity() {
            return Ok(vec![]);
        }

        let Some((request, token)) = self.queued.pop_front() else {
            return Ok(vec![]);
        };
        self.on_transaction_requested(request, token)
    }

    fn on_timeout_reached(&mut self, timeout: Timeout) -> Result {
        match timeout {
            Timeout::NonLifetime(timeout) => self.on_non_lifetime(timeout),
            Timeout::Retransmission(timeout) => self.on_retransmission(timeout),
            Timeout::ExchangeLifetime(timeout) => self.on_exchange_lifetime(timeout),
            Timeout::MaxTransmitWait(timeout) => self.on_max_transmit_wait(timeout),
            Timeout::NonRetransmission(_) => todo!(),
        }
    }

    fn on_max_transmit_wait(&mut self, timeout: MaxTransmitWaitTimeout) -> Result {
        let Some(Transaction::Confirmable(transaction)) = self
            .transaction_store
            .find_by_message_id(timeout.message_id())
        else {
            return Ok(vec![]);
        };

        match transaction.on_max_transmit_wait() {
            Ok(effects) => Ok(effects),
            Err(effects) => {
                self.transaction_store
                    .remove_by_message_id(timeout.message_id());

                Ok(effects)
            }
        }
    }

    fn on_exchange_lifetime(&mut self, timeout: ExchangeLifetimeTimeout) -> Result {
        self.on_lifetime(*timeout.message_id())
    }

    fn on_non_lifetime(&mut self, timeout: NonLifetimeTimeout) -> Result {
        self.on_lifetime(*timeout.message_id())
    }

    fn on_lifetime(&mut self, message_id: MessageId) -> Result {
        let mut effects = vec![];

        if let Some(transaction) = self.transaction_store.remove_by_message_id(&message_id) {
            effects.push(transaction.timeout());
        }

        self.message_id_store.release(message_id);

        effects.extend(self.dequeue_request()?);

        Ok(effects)
    }

    fn on_retransmission(&mut self, timeout: RetransmissionTimeout) -> Result {
        let Some(Transaction::Confirmable(transaction)) = self
            .transaction_store
            .find_mut_by_message_id(timeout.message_id())
        else {
            return Ok(vec![]);
        };

        match transaction.retransmit(timeout) {
            Ok(effects) => Ok(effects),
            Err(effects) => {
                self.transaction_store
                    .remove_by_message_id(timeout.message_id());
                Ok(effects)
            }
        }
    }

    fn on_transaction_requested(&mut self, request: NewRequest, token: Token) -> Result {
        if self.transaction_store.exists_by_token(&token) {
            return Err(Error::other("Token already exists"));
        }

        if self.at_capacity() {
            self.queued.push_back((request, token));
            return Ok(vec![]);
        }

        let transaction = Transaction::new(self.claim_message_id()?, token, request);

        let effects = transaction.initial_effects();

        self.transaction_store.add(transaction);

        Ok(effects)
    }

    fn on_response(&mut self, response: codec::Response) -> Result {
        let Some(transaction) = self.transaction_store.remove_by_token(&response.token()) else {
            return Ok(vec![]);
        };

        let mut effects = vec![];

        if response.reliability().is_confirmable() {
            effects.push(Effect::Transmit(
                Acknowledgement::new(response.message_id()).encode(),
            ))
        }

        effects.push(Effect::TransactionResolved(
            transaction.token().clone(),
            Ok(response.into()),
        ));

        Ok(effects)
    }

    fn on_piggyback(&mut self, piggyback: Piggyback) -> Result {
        self.on_response(piggyback.into())
    }

    fn on_acknowledgement(&mut self, acknowledgement: Acknowledgement) -> Result {
        let Some(transaction) = self
            .transaction_store
            .find_mut_by_message_id(&acknowledgement.message_id())
        else {
            return Ok(vec![]);
        };

        transaction.acknowledged();

        self.dequeue_request()
    }

    fn on_reset(&mut self, reset: Reset) -> Result {
        let Some(transaction) = self
            .transaction_store
            .remove_by_message_id(&reset.message_id())
        else {
            return Ok(vec![]);
        };

        let mut effects = vec![Effect::TransactionResolved(
            transaction.token().clone(),
            Err(response::Error::Reset),
        )];

        effects.extend(self.dequeue_request()?);

        Ok(effects)
    }
}

#[cfg(test)]
mod tests {

    use crate::codec::message::GetOptions;
    use crate::codec::Payload;
    use crate::protocol::get::Get;
    use crate::protocol::timeout::{
        ExchangeLifetimeTimeout, MaxTransmitWaitTimeout, NonLifetimeTimeout,
        NonRetransmissionTimeout, RetransmissionTimeout,
    };
    use crate::protocol::transmission_parameters::{
        ConfirmableParameters, InitialRetransmissionFactor, NonConfirmableParameters,
        ProbingRatePerSecond,
    };

    use message::{Piggyback, Reset};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::protocol::get;
    use crate::protocol::reliability::Reliability;
    use crate::protocol::transaction::con::ConfirmableTransaction;
    use crate::protocol::transaction::non_con::NonConfirmableTransacation;
    use crate::{
        codec::{
            code::response_code::Success, message, message_id::MessageId, token::Token,
            Acknowledgement, Options, Response, ResponseCode,
        },
        protocol::{
            effect::Effect, event::Event, message_id_store::MessageIdStore,
            new_request::NewRequest, processor::Processor, response,
        },
    };

    fn new_proccessor() -> Processor {
        let message_id_store = MessageIdStore::new(MessageId::from_value(0));
        Processor::new(message_id_store)
    }

    #[rstest]
    #[case(Reliability::NonConfirmable(NonConfirmableParameters::default()))]
    fn non_get_requested_without_retransmission(#[case] reliability: Reliability) {
        // Arrange
        let mut processor = new_proccessor();

        let token = Token::new().unwrap();
        let request = NewRequest::Get(get::Get {
            options: message::GetOptions::new(),
            reliability,
        });

        let expected_message = NonConfirmableTransacation::new(
            MessageId::from_value(0),
            token.clone(),
            request.clone(),
            NonConfirmableParameters::default(),
        )
        .request_data;

        //let expected_message = request.clone().encode();

        let event = Event::TransactionRequested(request, token);

        // Act
        let effects = processor.tick(event);

        // Assert
        let expected = Ok(vec![
            NonLifetimeTimeout::new(
                &MessageId::from_value(0),
                &NonConfirmableParameters::default(),
            )
            .into(),
            Effect::Transmit(expected_message),
        ]);
        assert_eq!(expected, effects)
    }

    #[rstest]
    fn non_get_requested_with_retransmission() {
        // Arrange
        let mut processor = new_proccessor();

        let token = Token::new().unwrap();
        let request = NewRequest::Get(get::Get {
            options: message::GetOptions::new(),
            reliability: Reliability::NonConfirmable(NonConfirmableParameters::new(
                Default::default(),
                Default::default(),
                Default::default(),
                Some(Default::default()),
            )),
        });

        let expected_message = NonConfirmableTransacation::new(
            MessageId::from_value(0),
            token.clone(),
            request.clone(),
            NonConfirmableParameters::default(),
        )
        .request_data;

        let event = Event::TransactionRequested(request, token);

        // Act
        let effects = processor.tick(event);

        // Assert
        let expected = Ok(vec![
            NonLifetimeTimeout::new(
                &MessageId::from_value(0),
                &NonConfirmableParameters::default(),
            )
            .into(),
            NonRetransmissionTimeout::new(
                &MessageId::from_value(0),
                expected_message.len(),
                &ProbingRatePerSecond::default(),
            )
            .into(),
            Effect::Transmit(expected_message),
        ]);
        assert_eq!(expected, effects)
    }

    #[rstest]
    fn con_get_requested() {
        // Arrange
        let reliability = Reliability::Confirmable(ConfirmableParameters::default(
            InitialRetransmissionFactor::new(0.5).unwrap(),
        ));
        let mut processor = new_proccessor();

        let token = Token::new().unwrap();
        let request = NewRequest::Get(get::Get {
            options: message::GetOptions::new(),
            reliability,
        });

        let transaction = ConfirmableTransaction::new(
            MessageId::from_value(0),
            token.clone(),
            request.clone(),
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        );

        let expected_message = transaction.clone().request_data;

        let event = Event::TransactionRequested(request, token);

        // Act
        let effects = processor.tick(event);

        // Assert
        let expected = Ok(vec![
            ExchangeLifetimeTimeout::new(
                MessageId::from_value(0),
                &transaction.transaction_parameters,
            )
            .into(),
            RetransmissionTimeout::new(
                MessageId::from_value(0),
                &transaction.transaction_parameters,
            )
            .into(),
            Effect::Transmit(expected_message),
        ]);
        assert_eq!(expected, effects)
    }

    #[rstest]
    fn con_get_acknowledged() {
        // Arrange
        let reliability = Reliability::Confirmable(ConfirmableParameters::default(
            InitialRetransmissionFactor::new(0.5).unwrap(),
        ));
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(get::Get {
            options: message::GetOptions::new(),
            reliability,
        });

        let event = Event::TransactionRequested(request, token.clone());
        processor.tick(event).unwrap();

        let response_message = Acknowledgement::new(message_id);

        // Act
        let response_bytes = response_message.encode();
        let effects = processor.tick(Event::DataReceived(response_bytes));

        // Assert
        let transcation = processor.transaction_store.find_by_token(&token).unwrap();

        assert_eq!(Ok(vec![]), effects);
        assert_eq!(true, transcation.is_acknowledged());
    }

    #[rstest]
    fn con_get_response() {
        // Arrange
        let reliability = Reliability::Confirmable(ConfirmableParameters::default(
            InitialRetransmissionFactor::new(0.5).unwrap(),
        ));
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(get::Get {
            options: message::GetOptions::new(),
            reliability,
        });

        let event = Event::TransactionRequested(request, token);

        let token = {
            let Event::TransactionRequested(_, token) = &event else {
                panic!("Should be requested")
            };
            token.clone()
        };

        processor.tick(event).unwrap();

        let acknowledge_message = Acknowledgement::new(message_id);

        let _effects = processor
            .tick(Event::DataReceived(acknowledge_message.encode()))
            .unwrap();

        let response_message = Response::new(
            message::Reliability::Confirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(1234),
            Options::new(),
            crate::codec::payload::Payload::from_value(
                "This is a cool message".as_bytes().to_vec(),
            ),
        );

        let message_id = response_message.message_id();
        let _payload = response_message.payload().clone().encode();

        let expected_response = response_message.clone();
        let effects = processor
            .tick(Event::DataReceived(response_message.encode()))
            .unwrap();

        // Act
        assert_eq!(
            vec![
                Effect::Transmit(Acknowledgement::new(message_id).encode()),
                Effect::TransactionResolved(token, Ok(expected_response.into()))
            ],
            effects
        );
    }

    #[rstest]
    fn retransmit_confirmable_transcation_until_max_retransmit_reached() {
        let mut processor = new_proccessor();

        // first transmission

        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        let effects = processor.tick(event).unwrap();

        let retransmission_timeout = RetransmissionTimeout::new(
            0.into(),
            &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        );

        assert_eq!(
            vec![
                ExchangeLifetimeTimeout::new(
                    0.into(),
                    &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap())
                )
                .into(),
                retransmission_timeout.into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // second transmission

        let effects = processor.tick(retransmission_timeout.into()).unwrap();
        let retransmission_timeout = retransmission_timeout.next();

        assert_eq!(
            vec![
                retransmission_timeout.into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // third transmission

        let effects = processor.tick(retransmission_timeout.into()).unwrap();
        let retransmission_timeout = retransmission_timeout.next();

        assert_eq!(
            vec![
                retransmission_timeout.into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // fourth transmission

        let effects = processor.tick(retransmission_timeout.into()).unwrap();
        let retransmission_timeout = retransmission_timeout.next();

        assert_eq!(
            vec![
                retransmission_timeout.into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // fifth transmission

        let effects = processor.tick(retransmission_timeout.into()).unwrap();
        let retransmission_timeout = retransmission_timeout.clone().next();

        assert_eq!(
            vec![
                retransmission_timeout.into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // attempt transmission but timeout due to `MAX_RETRANSMIT` reached

        let effects = processor.tick(retransmission_timeout.into()).unwrap();

        assert_eq!(
            vec![Effect::TransactionResolved(
                token,
                Err(response::Error::Timeout)
            ),],
            effects
        );
    }

    #[rstest]
    fn confirmable_transaction_received_reset() {
        let confirmable_parameters =
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap());
        let mut processor = new_proccessor();

        // transmission

        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        let effects = processor.tick(event).unwrap();

        assert_eq!(
            vec![
                ExchangeLifetimeTimeout::new(0.into(), &confirmable_parameters).into(),
                RetransmissionTimeout::new(0.into(), &confirmable_parameters).into(),
                Effect::Transmit(request.clone().encode(0.into(), token.clone()))
            ],
            effects
        );

        // receive reset

        let reset = Reset::from_message_id(0.into());
        let event = Event::DataReceived(reset.encode());
        let effects = processor.tick(event).unwrap();

        assert_eq!(
            vec![Effect::TransactionResolved(
                token,
                Err(response::Error::Reset)
            )],
            effects
        );
    }

    #[rstest]
    fn confirmable_message_sent_then_receives_acknowledgement() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let transaction = processor.transaction_store.find_by_token(&token).unwrap();
        assert_eq!(false, transaction.is_acknowledged());

        let acknowledgement = Acknowledgement::new(message_id);
        let event = Event::DataReceived(acknowledgement.encode());
        let effects = processor.tick(event).unwrap();

        let transaction = processor.transaction_store.find_by_token(&token).unwrap();
        assert_eq!(Vec::<Effect>::new(), effects);
        assert_eq!(true, transaction.is_acknowledged());
    }

    #[rstest]
    fn confirmable_message_acknowledged_then_receives_response() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let acknowledgement = Acknowledgement::new(message_id);
        let event = Event::DataReceived(acknowledgement.encode());
        processor.tick(event).unwrap();

        let response = Response::new(
            message::Reliability::Confirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(5),
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(response.clone().encode());
        let effects = processor.tick(event).unwrap();

        let response = self::response::Response {
            options: Options::new(),
            response_code: response.response_code(),
            payload: response.payload().clone(),
        };
        let acknowledgement = Acknowledgement::new(MessageId::from_value(5));
        let expected_effects = vec![
            Effect::Transmit(acknowledgement.encode()),
            Effect::TransactionResolved(token, Ok(response)),
        ];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_receives_reset() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let reset = Reset::from_message_id(message_id);
        let event = Event::DataReceived(reset.encode());
        let effects = processor.tick(event).unwrap();
        let expected_effects = vec![Effect::TransactionResolved(
            token,
            Err(response::Error::Reset),
        )];
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_receives_piggyback_response() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let piggyback = Piggyback::new(
            token.clone(),
            ResponseCode::Success(Success::Content),
            message_id,
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(piggyback.encode());
        let effects = processor.tick(event).unwrap();
        let response = self::response::Response {
            response_code: ResponseCode::Success(Success::Content),
            options: Options::new(),
            payload: Payload::empty(),
        };
        let expected_effects = vec![Effect::TransactionResolved(token, Ok(response))];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_is_timed_out_based_on_max_transmit_wait() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = MaxTransmitWaitTimeout::new(
            &message_id,
            &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        )
        .into();
        let effects = processor.tick(event).unwrap();
        let expected_effects = vec![Effect::TransactionResolved(
            token,
            Err(response::Error::Timeout),
        )];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_acknowledged_then_is_timed_out_based_on_max_transmit_wait() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();
        assert_eq!(
            true,
            processor
                .transaction_store
                .find_by_token(&token)
                .unwrap()
                .is_acknowledged()
        );

        let event = MaxTransmitWaitTimeout::new(
            &message_id,
            &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.5).unwrap()),
        )
        .into();
        let effects = processor.tick(event).unwrap();
        assert_eq!(1, processor.transaction_store.count());
        assert_eq!(Vec::<Effect>::new(), effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_is_timed_out_based_on_exchange_lifetime() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::TimeoutReached(
            ExchangeLifetimeTimeout::new(
                message_id,
                &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.0).unwrap()),
            )
            .into(),
        );
        let effects = processor.tick(event).unwrap();
        let expected_effects = vec![Effect::TransactionResolved(
            token,
            Err(response::Error::Timeout),
        )];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_acknowledged_then_is_timed_out_based_on_exchange_lifetime() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();
        assert_eq!(
            true,
            processor
                .transaction_store
                .find_by_token(&token)
                .unwrap()
                .is_acknowledged()
        );

        let event = Event::TimeoutReached(
            ExchangeLifetimeTimeout::new(
                message_id,
                &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.0).unwrap()),
            )
            .into(),
        );
        let effects = processor.tick(event).unwrap();
        let expected_effects = vec![Effect::TransactionResolved(
            token,
            Err(response::Error::Timeout),
        )];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_is_timed_out_based_on_transmission_counter() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let confirmable_parameters =
            ConfirmableParameters::default(InitialRetransmissionFactor::new(0.0).unwrap());
        let initial_retransmission_timeout =
            RetransmissionTimeout::new(message_id, &confirmable_parameters);

        let event = Event::TimeoutReached(initial_retransmission_timeout.clone().into());
        processor.tick(event).unwrap();

        let next_retranmission_timeout =
            RetransmissionTimeout::from_previous(initial_retransmission_timeout);
        let event = Event::TimeoutReached(next_retranmission_timeout.clone().into());
        processor.tick(event).unwrap();

        let next_retranmission_timeout =
            RetransmissionTimeout::from_previous(next_retranmission_timeout);
        let event = Event::TimeoutReached(next_retranmission_timeout.clone().into());
        processor.tick(event).unwrap();

        let next_retranmission_timeout =
            RetransmissionTimeout::from_previous(next_retranmission_timeout);
        let event = Event::TimeoutReached(next_retranmission_timeout.clone().into());
        processor.tick(event).unwrap();

        let next_retranmission_timeout =
            RetransmissionTimeout::from_previous(next_retranmission_timeout);
        let event = Event::TimeoutReached(next_retranmission_timeout.clone().into());
        let effects = processor.tick(event).unwrap();
        let expected_effects = vec![Effect::TransactionResolved(
            token,
            Err(response::Error::Timeout),
        )];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_acknowledged_then_ignore_retransmission() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();
        assert_eq!(
            true,
            processor
                .transaction_store
                .find_by_token(&token)
                .unwrap()
                .is_acknowledged()
        );

        let event = Event::TimeoutReached(
            RetransmissionTimeout::new(
                message_id,
                &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.0).unwrap()),
            )
            .into(),
        );
        let effects = processor.tick(event).unwrap();
        assert_eq!(Vec::<Effect>::new(), effects);
    }

    #[rstest]
    fn confirmable_message_sent_then_receives_response_before_acknowledgement() {
        let mut processor = new_proccessor();

        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();
        assert_eq!(
            false,
            processor
                .transaction_store
                .find_by_token(&token)
                .unwrap()
                .is_acknowledged()
        );

        let response = Response::new(
            message::Reliability::Confirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(5),
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(response.clone().encode());
        let effects = processor.tick(event).unwrap();

        let response = self::response::Response {
            options: Options::new(),
            response_code: response.response_code(),
            payload: response.payload().clone(),
        };
        let acknowledgement = Acknowledgement::new(MessageId::from_value(5));
        let expected_effects = vec![
            Effect::Transmit(acknowledgement.encode()),
            Effect::TransactionResolved(token, Ok(response)),
        ];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn confirmable_message_acknowledged_then_receives_non_confirmable_response() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();

        let response = Response::new(
            message::Reliability::NonConfirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(5),
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(response.clone().encode());
        let effects = processor.tick(event).unwrap();

        let response = self::response::Response {
            options: Options::new(),
            response_code: response.response_code(),
            payload: response.payload().clone(),
        };
        let expected_effects = vec![Effect::TransactionResolved(token, Ok(response))];
        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(expected_effects, effects);
    }

    #[rstest]
    fn transaction_resolved_and_before_exchange_lifetime_timeout() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();

        let response = Response::new(
            message::Reliability::NonConfirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(5),
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(response.clone().encode());
        processor.tick(event).unwrap();

        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(true, processor.message_id_store.is_claimed(&message_id));
    }

    #[rstest]
    fn transaction_resolved_and_after_exchange_lifetime_timeout() {
        let mut processor = new_proccessor();

        let message_id = MessageId::from_value(0);
        let token = Token::new().unwrap();
        let request = NewRequest::Get(Get {
            options: GetOptions::new(),
            reliability: Reliability::Confirmable(ConfirmableParameters::default(
                InitialRetransmissionFactor::new(0.5).unwrap(),
            )),
        });

        let event = Event::TransactionRequested(request.clone(), token.clone());
        processor.tick(event).unwrap();

        let event = Event::DataReceived(Acknowledgement::new(message_id).encode());
        processor.tick(event).unwrap();

        let response = Response::new(
            message::Reliability::NonConfirmable,
            token.clone(),
            ResponseCode::Success(Success::Content),
            MessageId::from_value(5),
            Options::new(),
            Payload::empty(),
        );
        let event = Event::DataReceived(response.clone().encode());
        processor.tick(event).unwrap();

        let event = Event::TimeoutReached(
            ExchangeLifetimeTimeout::new(
                message_id,
                &ConfirmableParameters::default(InitialRetransmissionFactor::new(0.0).unwrap()),
            )
            .into(),
        );
        processor.tick(event).unwrap();

        assert_eq!(0, processor.transaction_store.count());
        assert_eq!(false, processor.message_id_store.is_claimed(&message_id));
    }
}
