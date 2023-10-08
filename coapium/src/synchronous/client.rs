use std::{
    net::UdpSocket,
    sync::mpsc::{channel, Sender},
    thread::spawn,
};

use crate::{
    codec::{url::Endpoint, MessageId},
    protocol::{
        message_id_store::MessageIdStore,
        new_request::NewRequest,
        ping::{self, Ping},
        processor::Processor,
        response::{self, Response},
    },
    synchronous::system,
};

use super::system::{Command, System};

#[derive(Debug, Clone)]
pub struct Client {
    request_sender: Sender<Command>,
}

fn run_loop(mut system: System, message_id_store: MessageIdStore) -> Result<(), ()> {
    let mut processor = Processor::new(message_id_store);
    loop {
        let events = system.poll()?;
        let effects = events
            .into_iter()
            .map(|event| processor.tick(event))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ())?;

        let effects = effects.into_iter().flatten().collect();

        system.dispatch(effects)?;
    }
}

impl Client {
    pub fn new(endpoint: Endpoint) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let connect_address = format!(
            "{}:{}",
            endpoint.host,
            endpoint
                .port
                .map(|p| p.value())
                .unwrap_or(Default::default())
        );
        socket.connect(&connect_address).unwrap();

        let initial_message_id = MessageId::from_value(rand::random());
        let message_id_store = MessageIdStore::new(initial_message_id);

        let system = System::new(socket);
        let request_sender = system.get_sender();

        spawn(|| run_loop(system, message_id_store));

        Self { request_sender }
    }

    pub fn ping(&self, ping: Ping) -> Result<(), ping::Error> {
        let (sender, receiver) = channel();
        self.request_sender
            .send(Command::Ping(ping, sender))
            .expect("Failed to send to system");
        let (_token, receiver) = match receiver
            .recv()
            .expect("Failed to receive request accepted from system")
        {
            Ok((token, receiver)) => (token, receiver),
            _ => unreachable!(),
        };

        receiver
            .recv()
            .expect("Failed to receive from response from system")
    }

    pub fn execute(&self, request: NewRequest) -> Result<Response, response::Error> {
        let (sender, receiver) = System::new_request_channel();
        self.request_sender
            .send(Command::Request(request, sender))
            .expect("Failed to send to system");

        use system::Request::*;
        let (_token, receiver) = match receiver
            .recv()
            .expect("Failed to receive request accepted from system")
        {
            Accepted(token, receiver) => (token, receiver),
            _ => unreachable!(),
        };

        receiver
            .recv()
            .expect("Failed to receive from response from system")
    }
}
