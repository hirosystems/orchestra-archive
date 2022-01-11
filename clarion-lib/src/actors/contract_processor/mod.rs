use clarinet_lib::types::StacksTransactionData;
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};

#[derive(Clone, Debug)]
pub enum ContractProcessorMessage {
    ProcessChain,
    ProcessTransaction(StacksTransactionData),
    RollbackTransaction(StacksTransactionData),
    AddObserver(u8),
    RemoveObserver(u8),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct ContractProcessor {
    ctx: ComponentContext<Self>,
    observers: Vec<u8>,
}

impl ContractProcessor {
    pub fn new() -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            observers: vec![],
        }
    }
}

impl ComponentLifecycle for ContractProcessor {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ContractProcessor starting");

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
            ContractProcessorMessage::ProcessTransaction(tx) => {

            },
            ContractProcessorMessage::RollbackTransaction(tx) => {

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
