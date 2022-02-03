use crate::datastore::Datastore;
use clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;
use clarinet_lib::types::{BitcoinChainEvent, StacksChainEvent};

pub fn stacks_chain_event_handler(
    datastore: &dyn Datastore,
    contract_id: QualifiedContractIdentifier,
    chain_event: StacksChainEvent,
) {
    match chain_event {
        StacksChainEvent::ChainUpdatedWithBlock(block) => {}
        StacksChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {}
    }
}
