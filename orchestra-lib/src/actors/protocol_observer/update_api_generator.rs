use crate::datastore::Datastore;
use clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;
use clarinet_lib::types::StacksChainEvent;

pub fn stacks_chain_event_handler(
    datastore: &dyn Datastore,
    contract_id: QualifiedContractIdentifier,
    chain_event: StacksChainEvent,
) {
    match chain_event {
        StacksChainEvent::ChainUpdatedWithBlock(update) => {}
        StacksChainEvent::ChainUpdatedWithReorg(update) => {}
        StacksChainEvent::ChainUpdatedWithMicroblock(update) => {}
        StacksChainEvent::ChainUpdatedWithMicroblockReorg(update) => {}
    }
}
