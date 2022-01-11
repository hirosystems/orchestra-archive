use std::path::PathBuf;

pub trait Datastore {}
// InMemory Datastore
// OnDisk Datastore
// Remote Datastore

#[derive(Clone)]
pub enum StorageDriver {
    Filesystem(FilesystemConfig)
}

impl StorageDriver {

    pub fn filesystem(working_dir: PathBuf) -> StorageDriver {
        StorageDriver::Filesystem(FilesystemConfig {
            working_dir,
        })
    }
}

#[derive(Clone)]
pub struct FilesystemConfig {
    pub working_dir: PathBuf,
}
