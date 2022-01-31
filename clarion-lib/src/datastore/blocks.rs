use std::path::PathBuf;
use rocksdb::{DB, Options};
use super::StorageDriver;
use clarinet_lib::clarity_repl::clarity::util::hash::hex_bytes;

pub fn bitcoin_blocks_db_path(storage_driver: &StorageDriver) -> PathBuf {
    let mut working_dir = match storage_driver {
        StorageDriver::Filesystem(ref config) => config.working_dir.clone(),
    };
    working_dir.push("bitcoin");
    working_dir
}

pub fn stacks_blocks_db_path(storage_driver: &StorageDriver) -> PathBuf {
    let mut working_dir = match storage_driver {
        StorageDriver::Filesystem(ref config) => config.working_dir.clone(),
    };
    working_dir.push("stacks");
    working_dir
}

pub fn bitcoin_blocks_db_read(storage_driver: &StorageDriver) -> DB {
    let working_dir = bitcoin_blocks_db_path(storage_driver); 
    let options = Options::default();
    DB::open_for_read_only(&options, working_dir, true).unwrap()
}

pub fn bitcoin_blocks_db_write(storage_driver: &StorageDriver) -> DB {
    let working_dir = bitcoin_blocks_db_path(storage_driver); 
    let options = Options::default();
    DB::open_default(working_dir).unwrap()
}

pub fn stacks_blocks_db_read(storage_driver: &StorageDriver) -> DB {
    let working_dir = stacks_blocks_db_path(storage_driver); 
    let options = Options::default();
    DB::open_for_read_only(&options, working_dir, true).unwrap()
}

pub fn stacks_blocks_db_write(storage_driver: &StorageDriver) -> DB {
    let working_dir = stacks_blocks_db_path(storage_driver); 
    let options = Options::default();
    DB::open_default(working_dir).unwrap()
}

pub enum DBKey <'a> {
    FullAnalysis,
    Interface,
    Var(&'a str),
    VarEvent(&'a str, u64, u32),
    VarEventScanBlock(&'a str, u64),
    VarEventScan(&'a str),
    MapEntry(&'a str, &'a str),
    MapScan(&'a str),
    MapEvent(&'a str, u64, u32),
    MapEventScanBlock(&'a str, u64),
    MapEventScan(&'a str),
    FT(&'a str, &'a str),
    FTScan(&'a str),
    FTEvent(&'a str, u64, u32),
    FTEventScanBlock(&'a str, u64),
    FTEventScan(&'a str),
    NFT(&'a str, &'a str),
    NFTScan(&'a str),
    NFTEvent(&'a str, u64, u32),
    NFTEventScan(&'a str),
    NFTEventScanBlock(&'a str, u64),
}

pub fn db_key(key: DBKey, contract_id: &str) -> Vec<u8> {
    match key {
        DBKey::FullAnalysis => format!("{}::#analysis", contract_id).as_bytes().to_vec(),
        DBKey::Interface => format!("{}::#interface", contract_id).as_bytes().to_vec(),
        DBKey::Var(var) => format!("var::{}::{}", contract_id, var).as_bytes().to_vec(),
        DBKey::MapEntry(map, key) => {
            let mut prefix = format!("map::{}::{}@", contract_id, map).as_bytes().to_vec();
            let mut entry = hex_bytes(key).unwrap();
            prefix.append(&mut entry);
            prefix
        }
        DBKey::MapScan(map) => format!("map::{}::{}@", contract_id, map).as_bytes().to_vec(),
        DBKey::FT(asset_id, owner) => format!("ft::{}@{}", asset_id, owner).as_bytes().to_vec(),
        DBKey::FTScan(asset_id) => format!("ft::{}@", asset_id).as_bytes().to_vec(),
        DBKey::NFT(asset_id, key) => {
            let mut prefix = format!("nft::{}::id@", asset_id).as_bytes().to_vec();
            let mut entry = hex_bytes(key).unwrap();
            prefix.append(&mut entry);
            prefix
        }
        DBKey::NFTScan(asset_id) => format!("nft::{}::id@", asset_id).as_bytes().to_vec(),
        DBKey::VarEvent(var, block_index, event_index) => format!("var::{}::{}#events::{}/{}", contract_id, var, block_index, event_index).as_bytes().to_vec(),
        DBKey::VarEventScanBlock(var, block_index) => format!("var::{}::{}#events::{}/", contract_id, var, block_index).as_bytes().to_vec(),
        DBKey::VarEventScan(var) => format!("var::{}::{}#events::", contract_id, var).as_bytes().to_vec(),
        DBKey::MapEvent(map, block_index, event_index) => format!("map::{}::{}#events::{}/{}", contract_id, map, block_index, event_index).as_bytes().to_vec(),
        DBKey::MapEventScanBlock(map, block_index) => format!("map::{}::{}#events::{}/", contract_id, map, block_index).as_bytes().to_vec(),
        DBKey::MapEventScan(map) => format!("map::{}::{}#events::", contract_id, map).as_bytes().to_vec(),
        DBKey::FTEvent(asset_id, block_index, event_index) => format!("ft::{}#events::{}/{}", asset_id, block_index, event_index).as_bytes().to_vec(),
        DBKey::FTEventScanBlock(asset_id, block_index) => format!("ft::{}#events::{}/", asset_id, block_index).as_bytes().to_vec(),
        DBKey::FTEventScan(asset_id) => format!("ft::{}#events::", asset_id).as_bytes().to_vec(),
        DBKey::NFTEvent(asset_id, block_index, event_index) => format!("nft::{}#events::{}/{}", asset_id, block_index, event_index).as_bytes().to_vec(),
        DBKey::NFTEventScanBlock(asset_id, block_index) => format!("nft::{}#events::{}/", asset_id, block_index).as_bytes().to_vec(),
        DBKey::NFTEventScan(asset_id) => format!("nft::{}#events::", asset_id).as_bytes().to_vec(),
    }
}