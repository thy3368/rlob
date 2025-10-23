use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct Address {
    pub value: i32,
}

impl Address {
    pub fn new(value: i32) -> Address {
        Address { value }
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address {{ value: {} }}", self.value)
    }
}

pub trait Service {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>>;
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>>;
}

pub struct AddressServiceImpl {
    pub addressRepo: AddressRepoImpl,
}

impl Service for AddressServiceImpl {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>> {
        self.addressRepo.save(address)?;
        todo!()
    }
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>> {
        Ok(None)
    }
}

pub trait Repo: Send + Sync {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>>;
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>>;
}

pub struct AddressRepoImpl {}
impl Repo for AddressRepoImpl {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>> {
        Ok(None)
    }
}
