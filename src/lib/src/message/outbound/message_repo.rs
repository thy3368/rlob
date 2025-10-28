use crate::message::domain::message::{Command, CommandRepo};
use crate::multicase::outbound::udp_publisher::UdpMulticastPublisher;
use crate::unicase::outbound::tcp_client::TcpUnicastClient;
use crate::unicase::outbound::tcp_server::TcpUnicastServer;
use std::error::Error;

impl CommandRepo for UdpMulticastPublisher {
    fn send(&self, message: &Command) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn find_by_id(&self, id: &Command) -> Result<Option<Command>, Box<dyn Error>> {
        todo!()
    }
}

impl CommandRepo for TcpUnicastClient {
    fn send(&self, message: &Command) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, id: &Command) -> Result<Option<Command>, Box<dyn Error>> {
        todo!()
    }
}

impl CommandRepo for TcpUnicastServer {
    fn send(&self, message: &Command) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, id: &Command) -> Result<Option<Command>, Box<dyn Error>> {
        todo!()
    }
}
