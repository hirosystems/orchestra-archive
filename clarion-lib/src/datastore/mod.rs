use std::path::PathBuf;

pub trait Datastore {}
// InMemory Datastore
// OnDisk Datastore
// Remote Datastore

#[derive(Clone, Debug)]
pub enum StorageDriver {
    Filesystem(FilesystemConfig)
}

impl StorageDriver {

    pub fn filesystem(working_dir: PathBuf) -> StorageDriver {
        StorageDriver::Filesystem(FilesystemConfig {
            working_dir,
        })
    }

    pub fn tmpfs() -> StorageDriver {
        let mut working_dir = std::env::temp_dir();
        working_dir.push("clarion");
        StorageDriver::Filesystem(FilesystemConfig {
            working_dir
        })
    }

}

#[derive(Clone, Debug)]
pub struct FilesystemConfig {
    pub working_dir: PathBuf,
}
