use crate::message::domain::message::{Message, MessageRepo};
use crate::multicase::outbound::udp_publisher::UdpMulticastPublisher;
use crate::unicase::outbound::tcp_client::TcpUnicastClient;
use crate::unicase::outbound::tcp_server::TcpUnicastServer;
use std::error::Error;

impl MessageRepo for UdpMulticastPublisher {
    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn find_by_id(&self, id: &Message) -> Result<Option<Message>, Box<dyn Error>> {
        todo!()
    }
}

impl MessageRepo for TcpUnicastClient {
    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, id: &Message) -> Result<Option<Message>, Box<dyn Error>> {
        todo!()
    }
}

impl MessageRepo for TcpUnicastServer {
    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, id: &Message) -> Result<Option<Message>, Box<dyn Error>> {
        todo!()
    }
}
