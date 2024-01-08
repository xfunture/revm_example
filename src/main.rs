use anyhow::{Ok,Result};
// use ethers::middleware::gas_oracle::cache;
// use futures::io::Empty;
// use revm_primitives::{ruint::aliases::B160};
// use revm_primitives::alloy_primitives::address;

use ethers::middleware::gas_oracle::cache;
use revm::EVM;
use revm::db::CacheDB;
use revm::db::EmptyDB;
use std::sync::Arc;
use ethers::providers::Provider;
use revm::{db::EthersDB, Database};
use revm_primitives::U256;
use revm_primitives::alloy_primitives::Address;
const ALCHEMY_RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/cuT7e3X1csYwxDNQOI-l0QibriBl2CAC";


#[tokio::main]
async fn main() ->Result<()>{
    println!("revm example");
    let http_url = ALCHEMY_RPC_URL;
    let client = Provider::try_from(http_url)?;
    let client = Arc::new(client);

    let mut ethersdb = EthersDB::new(client.clone(),None).expect("create EthersDB error");


    let checksummed = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852";
    let pool_address = Address::parse_checksummed(checksummed, None).expect("valid checksum");

    // let pool_address = Address("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    let acc_info = ethersdb.basic(pool_address).unwrap().unwrap();

    // println!("acc_info:\n{:?}",acc_info);

    let slot = U256::from(8);
    let value = ethersdb.storage(pool_address, slot).unwrap();

    println!("storage value:{:?}",value);

    let mut cache_db = CacheDB::new(EmptyDB::default());
    cache_db.insert_account_info(pool_address, acc_info);
    cache_db.insert_account_storage(pool_address, slot, value).unwrap();

    let mut evm = EVM::new();
    evm.database(cache_db);


    Ok(())
}



