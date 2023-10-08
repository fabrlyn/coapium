use tokio::sync::mpsc::channel;
use tokio::{net::UdpSocket, sync::mpsc::UnboundedSender};

use crate::protocol::new_request::NewRequest;
use crate::protocol::ping::Ping;
use crate::protocol::{ping, response};
use crate::{
    asynchronous::system,
    codec::{message_id::MessageId, url::Endpoint},
    protocol::{message_id_store::MessageIdStore, processor::Processor},
};

use super::response::Response;
use super::system::{Command, System};

// TODO: Try this for diagnostics: https://github.com/tokio-rs/console

#[derive(Debug, Clone)]
pub struct Client {
    request_sender: UnboundedSender<Command>,
}

async fn run_loop(mut system: System, message_id_store: MessageIdStore) -> Result<(), ()> {
    let mut processor = Processor::new(message_id_store);
    loop {
        let event = system.poll().await?;
        let effects = processor.tick(event).map_err(|_| ())?;
        system.dispatch(effects).await?;
    }
}

impl Client {
    pub async fn new(endpoint: Endpoint) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let connect_address = format!(
            "{}:{}",
            endpoint.host,
            endpoint
                .port
                .map(|p| p.value())
                .unwrap_or(Default::default())
        );
        println!("{:?}", connect_address);
        socket.connect(&connect_address).await.unwrap();

        let initial_message_id = MessageId::from_value(rand::random());
        let message_id_store = MessageIdStore::new(initial_message_id);

        let system = System::new(socket);
        let request_sender = system.get_sender();

        tokio::spawn(async { run_loop(system, message_id_store).await });

        Self { request_sender }
    }

    pub async fn ping(&self, ping: Ping) -> Result<(), ping::Error> {
        let (sender, mut receiver) = channel(2);
        self.request_sender
            .send(Command::Ping(ping, sender))
            .expect("Failed to send to system");

        let (_token, mut receiver) = match receiver
            .recv()
            .await
            .expect("Failed to receive request accepted from system")
        {
            Ok((token, receiver)) => (token, receiver),
            _ => unreachable!(),
        };

        receiver
            .recv()
            .await
            .expect("Failed to receive from response from system")
    }

    pub async fn execute(&self, request: NewRequest) -> Result<Response, response::Error> {
        let (sender, mut receiver) = System::new_request_channel();
        self.request_sender
            .send(Command::Request(request, sender))
            .expect("Failed to send to system");

        use system::Request::*;
        let (_token, mut receiver) = match receiver
            .recv()
            .await
            .expect("Failed to receive request accepted from system")
        {
            Accepted(token, receiver) => (token, receiver),
            _ => unreachable!(),
        };

        receiver
            .recv()
            .await
            .expect("Failed to receive from response from system")
    }
}
