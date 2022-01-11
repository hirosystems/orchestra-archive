use std::collections::BTreeSet;

use clarinet_lib::types::StacksTransactionData;
use crate::types::ContractsObserverConfig;
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};

use crate::datastore::StorageDriver;

#[derive(Clone, Debug)]
pub enum ContractProcessorMessage {
    ProcessChain,
    ProcessTransactionsBatch(Vec<StacksTransactionData>),
    RollbackTransactionsBatch(Vec<StacksTransactionData>),
    AddObserver(ContractsObserverConfig),
    RemoveObserver(ContractsObserverConfig),
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractProcessorEvent {
    TransactionsBatchProcessed(String),
}

pub struct ContractProcessorPort;

impl Port for ContractProcessorPort {
    type Indication = ContractProcessorEvent;
    type Request = Never;
}

#[derive(ComponentDefinition)]
pub struct ContractProcessor {
    ctx: ComponentContext<Self>,
    contract_processor_port: ProvidedPort<ContractProcessorPort>,
    contract_id: String,
    storage_driver: StorageDriver,
}

ignore_requests!(ContractProcessorPort, ContractProcessor);

impl ContractProcessor {
    pub fn new(storage_driver: StorageDriver, contract_id: String) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            contract_processor_port: ProvidedPort::uninitialised(),
            storage_driver, 
            contract_id,
        }
    }

    // pub fn start_contracts_observer(&mut self, contract_id: String, config: &ContractsObserverConfig) {
    //     let system = self.ctx.system();
    //     let instance = system.create(ContractsObserver::new);
    //     system.start(&instance);
    //     // self.clarion_controllers.insert(pid.clone(), controller);
    //     // self.instances_pool.insert(pid, instance);
    // }
}

impl ComponentLifecycle for ContractProcessor {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ContractProcessor starting");

        // Retrieve contract from contracts store.
        // Retrieve ABI

        Handled::Ok
    }
}

impl Actor for ContractProcessor {
    type Message = ContractProcessorMessage;

    fn receive_local(&mut self, msg: ContractProcessorMessage) -> Handled {
        info!(self.ctx.log(), "ContractProcessor received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ContractProcessor")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ContractProcessorMessage::ProcessChain => {
            },
            ContractProcessorMessage::ProcessTransactionsBatch(tx) => {
                info!(self.ctx.log(), "ContractProcessor processed transaction batch");
                self.contract_processor_port.trigger(ContractProcessorEvent::TransactionsBatchProcessed(self.contract_id.clone()))
            },
            ContractProcessorMessage::RollbackTransactionsBatch(tx) => {

            },
            ContractProcessorMessage::AddObserver(observer) => {

            },
            ContractProcessorMessage::RemoveObserver(observer) => {

            },
            ContractProcessorMessage::Exit => {

            }
        };
        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}

// fn receive_local(&mut self, msg: BlockStoreManagerMessage) -> Handled {
//     info!(self.ctx.log(), "BlockStoreManager received message");

//     let tracer = opentelemetry_jaeger::new_pipeline()
//         .with_service_name("ContractProcessor")
//         .install_simple().unwrap();
//     let mut span = tracer.start("handle message");

//     match msg {
//         BlockStoreManagerMessage::ArchiveBitcoinBlock(block) => {
//             info!(self.log(), "BlockStoreManager will archive bitcoin block");
//             self.store_bitcoin_block(block);
//         },
//         BlockStoreManagerMessage::RollbackBitcoinBlocks(block_ids) => {
//             info!(self.log(), "BlockStoreManager will rollback bitcoin blocks");
//             self.delete_bitcoin_blocks(block_ids);
//         },
//         BlockStoreManagerMessage::ArchiveStacksBlock(block) => {
//             info!(self.log(), "BlockStoreManager will archive stacks block");
//             self.store_stacks_block(block);
//         },
//         BlockStoreManagerMessage::RollbackStacksBlocks(block_ids) => {
//             info!(self.log(), "BlockStoreManager will rollback stacks blocks");
//             self.delete_stacks_blocks(block_ids);
//         },
//         BlockStoreManagerMessage::Exit => {

//         },
//     };

//     span.end();
//     Handled::Ok
// }
