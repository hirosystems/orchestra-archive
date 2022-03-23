use crate::datastore::blocks;
use crate::datastore::StorageDriver;
use clarinet_lib::types::{
    BitcoinBlockData, BlockIdentifier, StacksBlockData, StacksMicroblockData,
    StacksMicroblocksTrail, StacksTransactionKind, TransactionIdentifier,
};
use kompact::prelude::*;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use rocksdb::DB;
use serde_json;

#[derive(Clone, Debug)]
pub enum BlockStoreManagerMessage {
    ArchiveBitcoinBlock(BitcoinBlockData),
    RollbackBitcoinBlocks(Vec<BlockIdentifier>),
    ArchiveStacksBlock(StacksBlockData, Option<StacksMicroblocksTrail>),
    RollbackStacksBlocks(Vec<BlockIdentifier>),
    ArchiveStacksMicroblock(StacksMicroblockData),
    RollbackStacksMicroblocks(Vec<BlockIdentifier>),
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
            storage_driver,
        }
    }

    pub fn store_bitcoin_block(&mut self, block: BitcoinBlockData) {
        let block_bytes = serde_json::to_vec(&block).expect("Unable to serialize block");
        let db = blocks::bitcoin_blocks_db_write(&self.storage_driver);
        db.put(
            format!("hash:{}", block.block_identifier.hash).as_bytes(),
            block_bytes,
        )
        .unwrap();
        db.put(
            block.block_identifier.index.to_be_bytes(),
            block.block_identifier.hash.as_bytes(),
        )
        .unwrap();
        db.put("tip".as_bytes(), block.block_identifier.index.to_be_bytes())
            .unwrap();
    }

    pub fn store_stacks_block(
        &mut self,
        block: StacksBlockData,
        anchored_trail: Option<StacksMicroblocksTrail>,
    ) {
        let block_bytes = serde_json::to_vec(&block).expect("Unable to serialize block");
        let db = blocks::stacks_blocks_db_write(&self.storage_driver);

        // Retrieve the parent block and append the transactions from the previous trail
        // note / todo: this choice could have an impact on re-orgs
        if let Some(anchored_trail) = anchored_trail {
            let bytes = db
                .get(&format!("hash:{}", block.parent_block_identifier.hash).as_bytes())
                .expect("Unable to hit contract storage")
                .expect("Unable to retrieve contract");
            let mut parent_block = serde_json::from_slice::<StacksBlockData>(&bytes)
                .expect("Unable to deserialize contract");

            for microblock in anchored_trail.microblocks.iter() {
                parent_block
                    .transactions
                    .append(&mut microblock.transactions.clone());
            }
            let parent_block_bytes =
                serde_json::to_vec(&parent_block).expect("Unable to serialize block");

            db.put(
                format!("hash:{}", block.parent_block_identifier.hash).as_bytes(),
                parent_block_bytes,
            )
            .unwrap();
        }

        for tx in block.transactions.iter() {
            match tx.metadata.kind {
                StacksTransactionKind::ContractDeployment(ref data) => {
                    let contract_instanciation = ContractInstanciation {
                        block_identifier: block.block_identifier.clone(),
                        tx_identifier: tx.transaction_identifier.clone(),
                        code: data.code.clone(),
                    };
                    let contract_instanciation_bytes = serde_json::to_vec(&contract_instanciation)
                        .expect("Unable to serialize block");
                    db.put(
                        data.contract_identifier.as_bytes(),
                        contract_instanciation_bytes,
                    )
                    .unwrap();
                }
                _ => {}
            };
        }
        db.put(
            format!("hash:{}", block.block_identifier.hash).as_bytes(),
            block_bytes,
        )
        .unwrap();
        db.put(
            block.block_identifier.index.to_be_bytes(),
            block.block_identifier.hash.as_bytes(),
        )
        .unwrap();
        db.put("tip".as_bytes(), block.block_identifier.index.to_be_bytes())
            .unwrap();
    }

    pub fn store_stacks_microblock(&mut self, microblock: StacksMicroblockData) {
        let block_bytes = serde_json::to_vec(&microblock).expect("Unable to serialize block");
        let db = blocks::stacks_blocks_db_write(&self.storage_driver);
        for tx in microblock.transactions.iter() {
            match tx.metadata.kind {
                StacksTransactionKind::ContractDeployment(ref data) => {
                    let contract_instanciation = ContractInstanciation {
                        block_identifier: microblock.parent_block_identifier.clone(),
                        tx_identifier: tx.transaction_identifier.clone(),
                        code: data.code.clone(),
                    };
                    let contract_instanciation_bytes = serde_json::to_vec(&contract_instanciation)
                        .expect("Unable to serialize block");
                    db.put(
                        data.contract_identifier.as_bytes(),
                        contract_instanciation_bytes,
                    )
                    .unwrap();
                }
                _ => {}
            };
        }
        db.put(
            format!("~:{}", microblock.block_identifier.index).as_bytes(),
            block_bytes,
        )
        .unwrap();
        db.put(
            "~tip".as_bytes(),
            microblock.block_identifier.index.to_be_bytes(),
        )
        .unwrap();
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

    pub fn delete_stacks_microblocks(&mut self, microblock_ids: Vec<BlockIdentifier>) {
        match self.storage_driver {
            StorageDriver::Filesystem(ref config) => {
                let mut path = config.working_dir.clone();
                path.push("stacks");
                let db = DB::open_default(path).unwrap();
                for block_id in microblock_ids.iter() {
                    // todo(lgalabru): remove contracts, update chain_tip
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
            .install_simple()
            .unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            BlockStoreManagerMessage::ArchiveBitcoinBlock(block) => {
                info!(
                    self.log(),
                    "BlockStoreManager will archive bitcoin block {}", block.block_identifier.index
                );
                self.store_bitcoin_block(block);
            }
            BlockStoreManagerMessage::RollbackBitcoinBlocks(block_ids) => {
                info!(self.log(), "BlockStoreManager will rollback bitcoin blocks");
                self.delete_bitcoin_blocks(block_ids);
            }
            BlockStoreManagerMessage::ArchiveStacksBlock(block, anchored_trail) => {
                info!(
                    self.log(),
                    "BlockStoreManager will archive stacks block {} - {}",
                    block.block_identifier.index,
                    block.block_identifier.hash
                );
                self.store_stacks_block(block, anchored_trail);
            }
            BlockStoreManagerMessage::RollbackStacksBlocks(block_ids) => {
                info!(self.log(), "BlockStoreManager will rollback stacks blocks");
                self.delete_stacks_blocks(block_ids);
            }
            BlockStoreManagerMessage::ArchiveStacksMicroblock(microblock) => {
                info!(
                    self.log(),
                    "BlockStoreManager will archive stacks microblock {}",
                    microblock.block_identifier.index
                );
                self.store_stacks_microblock(microblock);
            }
            BlockStoreManagerMessage::RollbackStacksMicroblocks(microblock_ids) => {
                info!(
                    self.log(),
                    "BlockStoreManager will rollback stacks microblocks"
                );
                self.delete_stacks_microblocks(microblock_ids);
            }
            BlockStoreManagerMessage::Exit => {}
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
