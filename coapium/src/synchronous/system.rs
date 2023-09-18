use std::{
    net::UdpSocket,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::spawn,
};

use log::error;

use crate::{
    codec::Token,
    protocol::{
        effect::{Effect, Effects, Timeout},
        event::Event,
        new_request::NewRequest,
        response::{self, Response},
        timeout::{
            ExchangeLifetimeTimeout, MaxTransmitWaitTimeout, NonLifetimeTimeout,
            NonRetransmissionTimeout, RetransmissionTimeout,
        },
        transaction::PATH_MTU,
    },
};

#[derive(Debug)]
pub enum Request {
    Accepted(Token, Receiver<Result<Response, response::Error>>),
    Rejected(),
}

#[derive(Debug)]
pub enum Command {
    Request(NewRequest, Sender<Request>),
    Cancel(Token),
    // Observe(...), maybe or Request is good enough
}

#[derive(Debug)]
pub struct System {
    requests: Vec<(Token, Sender<Result<Response, response::Error>>)>,
    command_receiver: Arc<Mutex<Receiver<Command>>>,
    command_sender: Sender<Command>,
    timeout_receiver: Arc<Mutex<Receiver<Timeout>>>,
    timeout_sender: Sender<Timeout>,
    incoming_socket_receiver: Arc<Mutex<Receiver<Vec<u8>>>>,
    udp_socket: Arc<UdpSocket>,
}

impl System {
    pub fn new_request_channel() -> (Sender<Request>, Receiver<Request>) {
        channel()
    }

    pub fn new(udp_socket: UdpSocket) -> Self {
        let (incoming_socket_sender, incoming_socket_receiver) = channel::<Vec<u8>>();

        let udp_socket = Arc::new(udp_socket);

        let socket_for_loop = udp_socket.clone();
        spawn(move || loop {
            let mut buffer = [0u8; PATH_MTU];

            let read = socket_for_loop.recv(&mut buffer).unwrap();
            if let Err(e) = incoming_socket_sender.send(buffer[..read].to_vec()) {
                println!("Failed to send data on incoming socket sender: {e:?}");
                return;
            }
        });

        let (command_sender, command_receiver) = channel();
        let (timeout_sender, timeout_receiver) = channel();

        Self {
            udp_socket,
            incoming_socket_receiver: Arc::new(Mutex::new(incoming_socket_receiver)),
            timeout_receiver: Arc::new(Mutex::new(timeout_receiver)),
            timeout_sender,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
            command_sender,
            requests: Default::default(),
        }
    }

    pub fn get_sender(&self) -> Sender<Command> {
        self.command_sender.clone()
    }

    fn on_command(&mut self, command: Command) -> Result<Event, ()> {
        match command {
            Command::Request(request, sender) => self.handle_request(request, sender),
            Command::Cancel(token) => self.handle_cancel(token),
        }
    }

    fn handle_cancel(&mut self, token: Token) -> Result<Event, ()> {
        self.requests.retain(|(t, _)| *t == token);
        Ok(Event::TransactionCanceled(token))
    }

    fn handle_request(
        &mut self,
        request: NewRequest,
        sender: Sender<Request>,
    ) -> Result<Event, ()> {
        let token = Token::new().map_err(|_| ())?;

        let (result_sender, result_receiver) = channel();
        if let Err(e) = sender.send(Request::Accepted(token.clone(), result_receiver)) {
            error!("Failed to send Request::Accepted to client: {e:?}");
            return Err(());
        }

        self.requests.push((token.clone(), result_sender));

        Ok(Event::TransactionRequested(request, token))
    }

    fn on_timeout(&mut self, timeout: Timeout) -> Result<Event, ()> {
        Ok(Event::TimeoutReached(timeout))
    }

    fn on_socket_data(&mut self, data: Vec<u8>) -> Result<Event, ()> {
        Ok(Event::DataReceived(data))
    }

    pub fn poll(&mut self) -> Result<Event, ()> {
        let command_receiver = self.command_receiver.clone();
        let command_receiver = &mut command_receiver.lock();
        //let command_future = command_receiver.recv();
        //pin!(command_future);

        let timeouts_receiver = self.timeout_receiver.clone();
        let timeouts_receiver = &mut timeouts_receiver.lock();
        //let timeouts_future = timeouts_receiver.recv();
        //pin!(timeouts_future);

        let socket_receiver = self.incoming_socket_receiver.clone();
        let socket_receiver = &mut socket_receiver.lock();
        //let socket_future = socket_receiver.recv();
        //pin!(socket_future);

        //select! {
        //    result = &mut command_future => {
        //        return self.on_command(result.ok_or(())?)
        //    }
        //    result = &mut timeouts_future => {
        //        return self.on_timeout(result.ok_or(())?)
        //    }
        //    result = &mut socket_future => {
        //        return self.on_socket_data(result.ok_or(())?)
        //    }
        //};
        todo!()
    }

    fn on_non_lifetime_timeout(&mut self, timeout: NonLifetimeTimeout) {
        let timeout_sender = self.timeout_sender.clone();
        //tokio::spawn(|| {
        //    sleep(*timeout.timeout());
        //    if let Err(e) = timeout_sender.send(Timeout::NonLifetime(timeout)) {
        //        error!("Failed to send non lifetime timeout: {e:?}");
        //    }
        //});
    }

    fn on_con_lifetime_timeout(&mut self, exchange_lifetime_timeout: ExchangeLifetimeTimeout) {
        //let timeout_sender = self.timeout_sender.clone();
        //tokio::spawn(move {
        //    sleep(*exchange_lifetime_timeout.timeout());
        //    if let Err(e) = timeout_sender.send(exchange_lifetime_timeout.into()) {
        //        error!("Failed to send exchange timeout: {e:?}");
        //    }
        //});
    }

    fn on_retransmission_timeout(&mut self, timeout: RetransmissionTimeout) {
        //let timeout_sender = self.timeout_sender.clone();
        //tokio::spawn(move {
        //    sleep(*timeout.timeout());
        //    if let Err(e) = timeout_sender.send(timeout.into()) {
        //        error!("Failed to send retransmission timeout: {e:?}");
        //    }
        //});
    }

    fn on_non_retransmission_timeout(&mut self, timeout: NonRetransmissionTimeout) {
        //let timeout_sender = self.timeout_sender.clone();
        //tokio::spawn(move {
        //    sleep(*timeout.timeout());
        //    if let Err(e) = timeout_sender.send(timeout.into()) {
        //        error!("Failed to send non retransmission timeout: {e:?}");
        //    }
        //});
    }

    fn on_max_transmit_wait(&mut self, timeout: MaxTransmitWaitTimeout) {
        //let timeout_sender = self.timeout_sender.clone();
        //tokio::spawn(||{
        //    sleep(*timeout.timeout());
        //    if let Err(e) = timeout_sender.send(timeout.into()) {
        //        error!("Failed to send max transmit wait timeout: {e:?}");
        //    }
        //});
    }

    fn on_create_timeout(&mut self, timeout: Timeout) {
        match timeout {
            Timeout::NonLifetime(timeout) => self.on_non_lifetime_timeout(timeout),
            Timeout::Retransmission(retransmission_timeout) => {
                self.on_retransmission_timeout(retransmission_timeout)
            }
            Timeout::ExchangeLifetime(exchange_lifetime_timeout) => {
                self.on_con_lifetime_timeout(exchange_lifetime_timeout)
            }
            Timeout::MaxTransmitWait(timeout) => self.on_max_transmit_wait(timeout),
            Timeout::NonRetransmission(timeout) => self.on_non_retransmission_timeout(timeout),
        }
    }

    fn remove_request_by_token(
        &mut self,
        token: &Token,
    ) -> Option<Sender<Result<Response, response::Error>>> {
        let Some(position) = self
            .requests
            .iter()
            .position(|(request_token, _)| request_token == token)
        else {
            return None;
        };

        Some(self.requests.swap_remove(position).1)
    }

    fn on_transaction_resolved(&mut self, token: Token, result: Result<Response, response::Error>) {
        let Some(request) = self.remove_request_by_token(&token) else {
            return;
        };
        if let Err(e) = request.send(result) {
            error!("Failed to send resolved transaction to requester: {e:?}");
        }
    }

    fn on_transmit(&mut self, data: Vec<u8>) {
        if let Err(e) = self.udp_socket.send(&data) {
            println!("Failed to send on udp socket: {e:?}");
        }
    }

    pub fn dispatch(&mut self, effects: Effects) -> Result<(), ()> {
        for effect in effects {
            match effect {
                Effect::CreateTimeout(timeout) => self.on_create_timeout(timeout),
                Effect::Transmit(data) => self.on_transmit(data),
                Effect::TransactionResolved(token, result) => {
                    self.on_transaction_resolved(token, result);
                }
            }
        }
        Ok(())
    }
}
