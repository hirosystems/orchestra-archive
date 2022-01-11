mod update_api_generator;
mod update_state_explorer;

pub use update_state_explorer::UpdateStateExplorer;

use clarinet_lib::types::StacksTransactionData;
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};

#[derive(Clone, Debug)]
pub enum ContractProcessorObserverMessage {
    ProcessChain,
    ProcessTransaction(StacksTransactionData),
    RollbackTransaction(StacksTransactionData),
    AddObserver(u8),
    RemoveObserver(u8),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct ContractProcessorObserver {
    ctx: ComponentContext<Self>,
    observers: Vec<u8>,
}

impl ContractProcessorObserver {
    pub fn new() -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            observers: vec![],
        }
    }
}

impl ComponentLifecycle for ContractProcessorObserver {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ContractProcessorObserver starting");

        Handled::Ok
    }
}

impl Actor for ContractProcessorObserver {
    type Message = ContractProcessorObserverMessage;

    fn receive_local(&mut self, msg: ContractProcessorObserverMessage) -> Handled {
        info!(self.ctx.log(), "ContractProcessorObserver received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ContractProcessorObserver")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ContractProcessorObserverMessage::ProcessChain => {

            },
            ContractProcessorObserverMessage::ProcessTransaction(tx) => {
                
            },
            ContractProcessorObserverMessage::RollbackTransaction(tx) => {

            },
            ContractProcessorObserverMessage::AddObserver(observer) => {

            },
            ContractProcessorObserverMessage::RemoveObserver(observer) => {

            },
            ContractProcessorObserverMessage::Exit => {

            }
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
