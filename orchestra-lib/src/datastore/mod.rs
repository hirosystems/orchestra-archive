pub mod blocks;
pub mod contracts;

use std::path::PathBuf;

pub trait Datastore {}
// InMemory Datastore
// OnDisk Datastore
// Remote Datastore

#[derive(Clone, Debug)]
pub enum StorageDriver {
    Filesystem(FilesystemConfig),
}

impl StorageDriver {
    pub fn filesystem(working_dir: PathBuf) -> StorageDriver {
        StorageDriver::Filesystem(FilesystemConfig { working_dir })
    }

    pub fn tmpfs() -> StorageDriver {
        let mut working_dir = std::env::temp_dir();
        working_dir.push("orchestra");
        StorageDriver::Filesystem(FilesystemConfig { working_dir })
    }
}

#[derive(Clone, Debug)]
pub struct FilesystemConfig {
    pub working_dir: PathBuf,
}

pub enum DataField {
    Var(String),
    Map(String),
    NonFungibleToken(String),
    FungibleToken(String),
}

pub fn get_contract_abi() {}

pub fn get_contract_analysis() {}

pub fn get_contract_data_field_history(field: &DataField) {}

pub fn get_contracts_observer() {}
