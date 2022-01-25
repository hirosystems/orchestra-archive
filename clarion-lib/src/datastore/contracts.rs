use std::path::PathBuf;

use rocksdb::{DB, Options};
use super::StorageDriver;

pub enum DBKey <'a> {
    FullAnalysis,
    Interface,
    Var(&'a str),
    Map(&'a str, &'a str),
    FT(&'a str, &'a str),
    NFT(&'a str, &'a str),
}

pub fn contract_db_path(storage_driver: &StorageDriver, contract_id: &str) -> PathBuf {
    let mut working_dir = match storage_driver {
        StorageDriver::Filesystem(ref config) => config.working_dir.clone(),
    };
    working_dir.push("contracts");
    working_dir.push(&contract_id);
    working_dir
}


pub fn contract_db_read(storage_driver: &StorageDriver, contract_id: &str) -> DB {
    let working_dir = contract_db_path(storage_driver, contract_id); 
    let options = Options::default();
    DB::open_for_read_only(&options, working_dir, true).unwrap()
}

pub fn contract_db_write(storage_driver: &StorageDriver, contract_id: &str) -> DB {
    let working_dir = contract_db_path(storage_driver, contract_id); 
    let options = Options::default();
    DB::open_default(working_dir).unwrap()
}


pub fn db_key(key: DBKey, contract_id: &str) -> Vec<u8> {
    match key {
        DBKey::FullAnalysis => format!("{}::@analysis", contract_id).as_bytes().to_vec(),
        DBKey::Interface => format!("{}::@interface", contract_id).as_bytes().to_vec(),
        DBKey::Var(var) => format!("{}::{}", contract_id, var).as_bytes().to_vec(),
        DBKey::Map(map, key) => format!("{}::{}::entry({})", contract_id, map, key).as_bytes().to_vec(),
        DBKey::FT(ft, owner) => format!("{}::owner({})", ft, owner).as_bytes().to_vec(),
        DBKey::NFT(nft, owner) => format!("{}::owner({})", nft, owner).as_bytes().to_vec(),
    }
}