use crate::domain::address::{Address, Repo};
use std::error::Error;

pub struct AddressDbRepo {}

impl AddressDbRepo {}

impl Repo for AddressDbRepo {
    fn save(&self, _alarm: &Address) -> Result<(), Box<dyn Error>> {
        // Placeholder implementation
        println!("AddressDbRepo: saving address (placeholder)");
        Ok(())
    }

    fn find_by_id(&self, _id: &Address) -> Result<Option<Address>, Box<dyn Error>> {
        // Placeholder implementation
        Ok(None)
    }
}
