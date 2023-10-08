use std::{
    io::ErrorKind,
    net::UdpSocket,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};

use log::error;

use crate::{
    codec::Token,
    protocol::{
        effect::{Effect, Effects, Timeout},
        event::{Event, Events},
        new_request::NewRequest,
        ping::{self, Ping},
        response::{self, Response},
        transaction::PATH_MTU,
    },
};

#[derive(Debug)]
pub enum Request {
    Accepted(Token, Receiver<Result<Response, response::Error>>),
    Rejected(),
}

#[derive(Debug)]
pub enum RequestSender {
    Ping(Sender<Result<(), ping::Error>>),
    Request(Sender<Result<Response, response::Error>>),
}

#[derive(Debug)]
pub enum Command {
    Request(NewRequest, Sender<Request>),
    Cancel(Token),
    Ping(
        Ping,
        Sender<Result<(Token, Receiver<Result<(), ping::Error>>), ()>>,
    ),
}

#[derive(Debug)]
pub struct System {
    requests: Vec<(Token, RequestSender)>,
    command_sender: Sender<Command>,
    command_receiver: Receiver<Command>,
    udp_socket: Arc<UdpSocket>,
    timeouts: Vec<(Instant, Timeout)>,
}

impl System {
    pub fn new_request_channel() -> (Sender<Request>, Receiver<Request>) {
        channel()
    }

    pub fn new(udp_socket: UdpSocket) -> Self {
        let udp_socket = Arc::new(udp_socket);

        let (command_sender, command_receiver) = channel();

        Self {
            udp_socket,
            command_sender,
            command_receiver,
            requests: Default::default(),
            timeouts: vec![],
        }
    }

    pub fn get_sender(&self) -> Sender<Command> {
        self.command_sender.clone()
    }

    fn on_command(&mut self, command: Command) -> Result<Event, ()> {
        match command {
            Command::Request(request, sender) => self.handle_request(request, sender),
            Command::Cancel(token) => self.handle_cancel(token),
            Command::Ping(ping, sender) => self.ping(ping, sender),
        }
    }

    fn handle_cancel(&mut self, token: Token) -> Result<Event, ()> {
        self.requests.retain(|(t, _)| *t == token);
        Ok(Event::TransactionCanceled(token))
    }

    fn ping(
        &mut self,
        ping: Ping,
        sender: Sender<Result<(Token, Receiver<Result<(), ping::Error>>), ()>>,
    ) -> Result<Event, ()> {
        let token = Token::new().map_err(|_| ())?;

        let (result_sender, result_receiver) = channel();
        if let Err(e) = sender.send(Ok((token.clone(), result_receiver))) {
            error!("Failed to send Request::Accepted to client: {e:?}");
            return Err(());
        }

        self.requests
            .push((token.clone(), RequestSender::Ping(result_sender)));

        Ok(Event::TransactionRequested(NewRequest::Ping(ping), token))
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

        self.requests
            .push((token.clone(), RequestSender::Request(result_sender)));

        Ok(Event::TransactionRequested(request, token))
    }

    pub fn poll(&mut self) -> Result<Events, ()> {
        let mut events = vec![];

        let mut buffer = [0u8; PATH_MTU];
        let read = self.udp_socket.recv(&mut buffer);

        match read {
            Ok(read) => {
                events.push(Event::DataReceived(buffer[..read].to_vec()));
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    return Err(());
                }
            }
        }

        let now = Instant::now();
        while let Some(index) = self
            .timeouts
            .iter()
            .position(|(timeout_at, _)| now >= *timeout_at)
        {
            let timeout = self.timeouts.swap_remove(index).1;
            events.push(Event::TimeoutReached(timeout));
        }

        match self.command_receiver.try_recv() {
            Ok(command) => {
                events.push(self.on_command(command)?);
            }
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => {}
                std::sync::mpsc::TryRecvError::Disconnected => return Err(()),
            },
        }

        Ok(events)
    }

    fn on_create_timeout(&mut self, timeout: Timeout) {
        let timeout_at = Instant::now() + *timeout.duration();
        self.timeouts.push((timeout_at, timeout))
    }

    fn remove_request_by_token(&mut self, token: &Token) -> Option<RequestSender> {
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

        match request {
            RequestSender::Ping(sender) => Self::on_ping_resolved(sender, result),
            RequestSender::Request(sender) => Self::on_request_resolved(sender, result),
        }
    }

    fn on_request_resolved(
        sender: Sender<Result<Response, response::Error>>,
        result: Result<Response, response::Error>,
    ) {
        if let Err(e) = sender.send(result) {
            error!("Failed to send resolved transaction to requester: {e:?}");
        }
    }

    fn on_ping_resolved(
        sender: Sender<Result<(), ping::Error>>,
        result: Result<Response, response::Error>,
    ) {
        if let Err(e) = sender.send(ping::into_result(result)) {
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
