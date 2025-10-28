use crate::multicase::outbound::udp_publisher::UdpMulticastPublisher;
use crate::unicase::outbound::tcp_client::TcpUnicastClient;
use crate::unicase::outbound::tcp_server::TcpUnicastServer;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct Message {
    pub value: i32,
}

impl Message {
    pub fn new(value: i32) -> Message {
        Message { value }
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message {{ value: {} }}", self.value)
    }
}

pub trait MessageRepo: Send + Sync + Sized {
    fn send(&self, message: &Message) -> Result<(), Box<dyn Error>>;
    fn find_by_id(&self, id: &Message) -> Result<Option<Message>, Box<dyn Error>>;
}
