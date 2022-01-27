use std::path::PathBuf;

use rocksdb::{DB, Options};
use super::StorageDriver;
use clarinet_lib::clarity_repl::clarity::util::hash::hex_bytes;

pub enum DBKey <'a> {
    FullAnalysis,
    Interface,
    Var(&'a str),
    MapEntry(&'a str, &'a str),
    MapScan(&'a str),
    FT(&'a str, &'a str),
    FTScan(&'a str),
    NFT(&'a str, &'a str),
    NFTScan(&'a str),
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
        DBKey::Var(var) => format!("var::{}::{}", contract_id, var).as_bytes().to_vec(),
        DBKey::MapEntry(map, key) => {
            let mut prefix = format!("map::{}::{}@", contract_id, map).as_bytes().to_vec();
            let mut entry = hex_bytes(key).unwrap();
            prefix.append(&mut entry);
            prefix
        }
        DBKey::MapScan(map) => format!("map::{}::{}", contract_id, map).as_bytes().to_vec(),
        DBKey::FT(asset_id, owner) => format!("ft::{}@{}", asset_id, owner).as_bytes().to_vec(),
        DBKey::FTScan(asset_id) => format!("ft::{}@", asset_id).as_bytes().to_vec(),
        DBKey::NFT(asset_id, key) => {
            let mut prefix = format!("nft::{}::id@", asset_id).as_bytes().to_vec();
            let mut entry = hex_bytes(key).unwrap();
            prefix.append(&mut entry);
            prefix
        }
        DBKey::NFTScan(asset_id) => format!("nft::{}::id@", asset_id).as_bytes().to_vec(),
    }
}