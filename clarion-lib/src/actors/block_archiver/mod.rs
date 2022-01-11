use clarinet_lib::types::{StacksBlockData, BitcoinBlockData, BlockIdentifier};
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};

#[derive(Clone, Debug)]
pub enum BlockArchiverMessage {
    ArchiveBitcoinBlock(BitcoinBlockData),
    RollbackBitcoinBlocks(Vec<BlockIdentifier>),
    ArchiveStacksBlock(StacksBlockData),
    RollbackStacksBlocks(Vec<BlockIdentifier>),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct BlockArchiver {
    ctx: ComponentContext<Self>,
}

impl BlockArchiver {
    pub fn BlockArchiver() -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
        }
    }
}

impl ComponentLifecycle for BlockArchiver {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "BlockArchiver starting");

        Handled::Ok
    }
}

impl Actor for BlockArchiver {
    type Message = BlockArchiverMessage;

    fn receive_local(&mut self, msg: BlockArchiverMessage) -> Handled {
        info!(self.ctx.log(), "BlockArchiver received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ContractProcessor")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            BlockArchiverMessage::ArchiveBitcoinBlock(block) => {

            },
            BlockArchiverMessage::RollbackBitcoinBlocks(blocks) => {

            },
            BlockArchiverMessage::ArchiveStacksBlock(block) => {

            },
            BlockArchiverMessage::RollbackStacksBlocks(blocks) => {

            },
            BlockArchiverMessage::Exit => {

            },
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
