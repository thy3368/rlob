use lib::exchange::domain::address::{AddressRepoImpl, AddressServiceImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Binance BTC/USDT Real-time Price Monitor ===\n");

    let repo = AddressRepoImpl {};
    let _service = AddressServiceImpl { address_repo: repo };
    Ok(())
}
