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

pub struct AddressServiceImpl<T: Repo> {
    pub address_repo: T,
}

impl<T: Repo> Service for AddressServiceImpl<T> {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>> {
        self.address_repo.save(address)?;
        Ok(())
    }
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>> {
        self.address_repo.find_by_id(id)
    }
}

pub trait Repo: Send + Sync + Sized {
    fn save(&self, address: &Address) -> Result<(), Box<dyn Error>>;
    fn find_by_id(&self, id: &Address) -> Result<Option<Address>, Box<dyn Error>>;
}

pub struct AddressRepoImpl {}
impl Repo for AddressRepoImpl {
    fn save(&self, _address: &Address) -> Result<(), Box<dyn Error>> {
        todo!()
    }
    fn find_by_id(&self, _id: &Address) -> Result<Option<Address>, Box<dyn Error>> {
        Ok(None)
    }
}
