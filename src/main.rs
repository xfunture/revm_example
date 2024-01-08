use anyhow::{Ok,Result};
// use ethers::middleware::gas_oracle::cache;
// use futures::io::Empty;
// use revm_primitives::{ruint::aliases::B160};
// use revm_primitives::alloy_primitives::address;

use ethers_core::abi::parse_abi;
use ethers_contract::BaseContract;
use revm_primitives::Bytecode;
use revm::EVM;
use revm::db::CacheDB;
use revm::db::EmptyDB;
use revm_primitives::ExecutionResult;
use revm_primitives::TransactTo;
use revm_primitives::Output;
use revm_primitives::alloy_primitives::B160;
use std::str::FromStr;
use std::sync::Arc;
use ethers::providers::Provider;
use revm::{db::EthersDB, Database};
use revm_primitives::U256;
use revm_primitives::B256;
use ethers_core::types::U64;
use revm_primitives::alloy_primitives::Address;


/*
use anyhow::{Ok, Result};
use bytes::Bytes;
use ethers_contract::BaseContract;
use ethers_core::abi::parse_abi;
use ethers_providers::{Http, Provider};
use revm::{
    db::{CacheDB, EmptyDB, EthersDB},
    primitives::{ExecutionResult, Output, TransactTo, B160, U256 as rU256},
    Database, EVM,
};
use std::{str::FromStr, sync::Arc};
 */
const ALCHEMY_RPC_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/cuT7e3X1csYwxDNQOI-l0QibriBl2CAC";



/*
 *  1.创建Http provider，用于连接以太坊节点，这里用的Alchemy 提供的节点
 *  2.创建EthersDB,该EthersDB用于缓存以太坊状态数据,通过EthersDB 可以获取以太坊数据
 *  3.创建一个CacheDB,将EthersDB 账户存储值注入到CacheDB,这样可以覆盖实际状态值，从而更容易在自定义环境中
 *     测试我们的交易
 *  4.通过EVM执行交易
 */



 pub struct AccountInfo{
    pub balance:U256,
    pub nonce:U64,
    pub code_hash:B256,
    pub code:Option<Bytecode>,
 }


#[tokio::main]
async fn main() ->Result<()>{
    // println!("revm example");
    let http_url = ALCHEMY_RPC_URL;
    let client = Provider::try_from(http_url)?;
    let client = Arc::new(client);

    let mut ethersdb = EthersDB::new(client.clone(),None).expect("create EthersDB error");


    //WETH-USDT Uniswap V2 pool
    let checksummed = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852";
    let pool_address = Address::parse_checksummed(checksummed, None).expect("valid checksum");

    let acc_info = ethersdb.basic(pool_address).unwrap().unwrap();

    // println!("acc_info:\n{:?}",acc_info);

    let slot = U256::from(8);
    let value = ethersdb.storage(pool_address, slot).unwrap();

    println!("storage value:{:?}",value);

    let mut cache_db = CacheDB::new(EmptyDB::default());
    cache_db.insert_account_info(pool_address, acc_info);
    cache_db.insert_account_storage(pool_address, slot, value).unwrap();


    //1.创建一个EVM实例，并且设置了一个cache_db
    //2.创建一个合约函数的abi(application binary interface)
    //3.运行交易

    let mut evm = EVM::new();
    evm.database(cache_db);

    let pool_contract = BaseContract::from(parse_abi(
        &["function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)",
        ])?
    );

    let encoded = pool_contract.encode("getReserves",())?;

    evm.env.tx.caller = Address::parse_checksummed("0x0000000000000000000000000000000000000000", None).expect("address error");
    evm.env.tx.transact_to = TransactTo::Call(pool_address);
    evm.env.tx.data = revm_primitives::Bytes(encoded.0);
    evm.env.tx.value = U256::ZERO;

    let ref_tx = evm.transact_ref().unwrap();
    let result = ref_tx.result;

    let value = match result{
        ExecutionResult::Success{output,..} => match output{
            Output::Call(value) => Some(value),
            _ => None,
        },
        _=> None,
    };
    println!("value:{:?}",value);

    let (reserve0,reservel,ts):(u128,u128,u32) = pool_contract.decode_output("getReserves",value.unwrap())?;

    println!("{:?} {:?} {:?}",reserve0,reservel,ts);

    Ok(())
}



