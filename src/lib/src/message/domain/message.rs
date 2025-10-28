use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct Command {
    pub function_id: String,
    pub params: Vec<Value>,
    pub value: i32,
}

impl Command {
    pub fn new(value: i32) -> Command {
        Command {
            function_id: "".to_string(),
            params: vec![],
            value,
        }
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message {{ value: {} }}", self.value)
    }
}

pub trait CommandService: Send + Sync + Sized {}
pub trait CommandRepo: Send + Sync + Sized {
    fn send(&self, message: &Command) -> Result<(), Box<dyn Error>>;
    fn find_by_id(&self, id: &Command) -> Result<Option<Command>, Box<dyn Error>>;
}
