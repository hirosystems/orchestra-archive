mod update_api_generator;
mod update_state_explorer;

pub use update_state_explorer::UpdateStateExplorer;

use clarinet_lib::types::StacksTransactionData;
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};

use crate::datastore::StorageDriver;
use crate::types::ContractsObserverConfig;

#[derive(Clone, Debug)]
pub enum ContractsObserverMessage {
    ProcessChain,
    ProcessTransaction(StacksTransactionData),
    RollbackTransaction(StacksTransactionData),
    AddObserver(u8),
    RemoveObserver(u8),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct ContractsObserver {
    ctx: ComponentContext<Self>,
    config: ContractsObserverConfig,
    storage_driver: StorageDriver,
}

impl ContractsObserver {
    pub fn new(storage_driver: StorageDriver, config: ContractsObserverConfig) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            storage_driver,
            config,
        }
    }
}

impl ComponentLifecycle for ContractsObserver {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ContractsObserver starting");

        Handled::Ok
    }
}

// impl Provide<ContractProcessorPort> for ContractsObserver {
//     fn handle(&mut self, event: ContractProcessorEvent) -> Handled {
//         match event {
//             ContractProcessorEvent::MapUpdated => {
//                 info!(self.log(), "Map updated");
//         		Handled::Ok
//             }
//             ContractProcessorEvent::VarUpdated => {
//                 info!(self.log(), "Var updated");
//         		Handled::Ok
//             }
//         }
//     }
// }

impl Actor for ContractsObserver {
    type Message = ContractsObserverMessage;

    fn receive_local(&mut self, msg: ContractsObserverMessage) -> Handled {
        info!(self.ctx.log(), "ContractsObserver received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ContractsObserver")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ContractsObserverMessage::ProcessChain => {

            },
            ContractsObserverMessage::ProcessTransaction(tx) => {
                
            },
            ContractsObserverMessage::RollbackTransaction(tx) => {

            },
            _ => {}
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}