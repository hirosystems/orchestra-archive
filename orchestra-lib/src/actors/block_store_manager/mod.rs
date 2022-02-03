use clarinet_lib::types::{StacksBlockData, BitcoinBlockData, BlockIdentifier, StacksTransactionKind, TransactionIdentifier};
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use crate::datastore::StorageDriver;
use rocksdb::{DB};
use serde_json;
use crate::datastore::blocks;

#[derive(Clone, Debug)]
pub enum BlockStoreManagerMessage {
    ArchiveBitcoinBlock(BitcoinBlockData),
    RollbackBitcoinBlocks(Vec<BlockIdentifier>),
    ArchiveStacksBlock(StacksBlockData),
    RollbackStacksBlocks(Vec<BlockIdentifier>),
    Exit,
}

use serde::{self, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContractInstanciation {
    /// Also known as the block height.
    pub block_identifier: BlockIdentifier,
    pub tx_identifier: TransactionIdentifier,
    pub code: String,
}


#[derive(ComponentDefinition)]
pub struct BlockStoreManager {
    ctx: ComponentContext<Self>,
    storage_driver: StorageDriver,
}

impl BlockStoreManager {
    pub fn new(storage_driver: StorageDriver) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            storage_driver
        }
    }

    pub fn store_bitcoin_block(&mut self, block: BitcoinBlockData) {
        let block_bytes = serde_json::to_vec(&block).expect("Unable to serialize block");
        let db = blocks::bitcoin_blocks_db_write(&self.storage_driver);
        db.put(format!("hash:{}", block.block_identifier.hash).as_bytes(), block_bytes).unwrap();
        db.put(block.block_identifier.index.to_be_bytes(), block.block_identifier.hash.as_bytes()).unwrap();
        db.put("tip".as_bytes(), block.block_identifier.index.to_be_bytes()).unwrap();
}

    pub fn store_stacks_block(&mut self, block: StacksBlockData) {
        let block_bytes = serde_json::to_vec(&block).expect("Unable to serialize block");
        let db = blocks::stacks_blocks_db_write(&self.storage_driver);
        for tx in block.transactions.iter() {
            match tx.metadata.kind {
                StacksTransactionKind::ContractDeployment(ref data) => {
                    let contract_instanciation = ContractInstanciation {
                        block_identifier: block.block_identifier.clone(),
                        tx_identifier: tx.transaction_identifier.clone(),
                        code: data.code.clone()
                    };
                    let contract_instanciation_bytes = serde_json::to_vec(&contract_instanciation).expect("Unable to serialize block");
                    db.put(data.contract_identifier.as_bytes(), contract_instanciation_bytes).unwrap();
                }
                _ => {}
            };
        }
        db.put(format!("hash:{}", block.block_identifier.hash).as_bytes(), block_bytes).unwrap();
        db.put(block.block_identifier.index.to_be_bytes(), block.block_identifier.hash.as_bytes()).unwrap();
        db.put("tip".as_bytes(), block.block_identifier.index.to_be_bytes()).unwrap();
    }

    pub fn delete_bitcoin_blocks(&mut self, block_ids: Vec<BlockIdentifier>) {
        match self.storage_driver {
            StorageDriver::Filesystem(ref config) => {
                let mut path = config.working_dir.clone();
                path.push("bitcoin");
                let db = DB::open_default(path).unwrap();
                for block_id in block_ids.iter() {
                    db.delete(block_id.hash.as_bytes()).unwrap();
                }
            }
        }
    }

    pub fn delete_stacks_blocks(&mut self, block_ids: Vec<BlockIdentifier>) {
        match self.storage_driver {
            StorageDriver::Filesystem(ref config) => {
                let mut path = config.working_dir.clone();
                path.push("stacks");
                let db = DB::open_default(path).unwrap();
                for block_id in block_ids.iter() {
                    // todo(lgalabru): remove contracts
                    db.delete(block_id.hash.as_bytes()).unwrap();
                }
            }
        }
    }
}

impl ComponentLifecycle for BlockStoreManager {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "BlockStoreManager starting");

        Handled::Ok
    }
}

impl Actor for BlockStoreManager {
    type Message = BlockStoreManagerMessage;

    fn receive_local(&mut self, msg: BlockStoreManagerMessage) -> Handled {
        info!(self.ctx.log(), "BlockStoreManager received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("BlockStoreManager")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            BlockStoreManagerMessage::ArchiveBitcoinBlock(block) => {
                info!(self.log(), "BlockStoreManager will archive bitcoin block");
                self.store_bitcoin_block(block);
            },
            BlockStoreManagerMessage::RollbackBitcoinBlocks(block_ids) => {
                info!(self.log(), "BlockStoreManager will rollback bitcoin blocks");
                self.delete_bitcoin_blocks(block_ids);
            },
            BlockStoreManagerMessage::ArchiveStacksBlock(block) => {
                info!(self.log(), "BlockStoreManager will archive stacks block");
                self.store_stacks_block(block);
            },
            BlockStoreManagerMessage::RollbackStacksBlocks(block_ids) => {
                info!(self.log(), "BlockStoreManager will rollback stacks blocks");
                self.delete_stacks_blocks(block_ids);
            },
            BlockStoreManagerMessage::Exit => {

            },
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
