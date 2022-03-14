use crate::actors::{
    BlockStoreManager, BlockStoreManagerMessage, ContractProcessor, ContractProcessorMessage,
    ProtocolObserver, ProtocolObserverMessage,
};
use crate::datastore::StorageDriver;
use crate::types::{
    BitcoinPredicate, FieldValues, FieldValuesRequest, ProtocolObserverConfig, ProtocolObserverId,
    ProtocolRegistration, StacksChainPredicates, TriggerId,
};
use clarinet_lib::clarity_repl::clarity::analysis::ContractAnalysis;
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::ContractInterface;
use clarinet_lib::clarity_repl::clarity::diagnostic::Diagnostic;
use clarinet_lib::clarity_repl::repl::ast::ContractAST;
use clarinet_lib::types::{
    BitcoinBlockData, BitcoinChainEvent, StacksBlockData, StacksChainEvent, StacksTransactionData,
    StacksTransactionReceipt, BlockIdentifier,
};
use kompact::prelude::*;
use rocksdb::DB;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::Sender;

use opentelemetry::trace::Tracer;
use opentelemetry::{global, trace::Span};

use super::contract_processor::{ContractProcessorEvent, ContractProcessorPort};

use super::protocol_observer::{ProtocolObserverEvent, ProtocolObserverPort};

#[derive(Clone, Debug)]
pub enum OrchestraSupervisorMessage {
    RegisterProtocolObserver(ProtocolObserverConfig),
    GetProtocolInterfaces(ProtocolObserverId, Sender<ProtocolRegistration>),
    ProcessStacksChainEvent(StacksChainEvent),
    ProcessBitcoinChainEvent(BitcoinChainEvent),
    GetFieldValues(FieldValuesRequest),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct OrchestraSupervisor {
    ctx: ComponentContext<Self>,
    active_contracts_processors: HashMap<String, ActorRef<ContractProcessorMessage>>,
    active_protocol_observers: HashMap<ProtocolObserverId, ActorRef<ProtocolObserverMessage>>,
    contracts_processors_subscriptions: BTreeMap<String, BTreeSet<ProtocolObserverId>>,
    block_store_manager: Option<ActorRef<BlockStoreManagerMessage>>, // Todo: switch to event instead
    registered_contracts: HashSet<String>,
    storage_driver: StorageDriver,
    stacks_predicates: StacksChainPredicates,
    contract_processor_port: RequiredPort<ContractProcessorPort>,
    protocol_observer_port: RequiredPort<ProtocolObserverPort>,
    bitcoin_predicates: HashMap<BitcoinPredicate, Vec<TriggerId>>,
    trigger_history: VecDeque<(String, HashSet<TriggerId>)>,
}

// ignore_indications!(SetOffset, DynamicManager);
// ignore_indications!(SetScale, DynamicManager);
// ignore_lifecycle!(OrchestraSupervisor);

impl ComponentLifecycle for OrchestraSupervisor {
    fn on_start(&mut self) -> Handled {
        info!(self.log(), "OrchestraSupervisor starting");

        // Ensure that we have access to storage by opening early connections
        match self.storage_driver {
            StorageDriver::Filesystem(ref config) => {
                let mut bitcoin_path = config.working_dir.clone();
                bitcoin_path.push("bitcoin");
                let _db = DB::open_default(bitcoin_path).unwrap();
                let mut stacks_path = config.working_dir.clone();
                stacks_path.push("stacks");
                let _db = DB::open_default(stacks_path).unwrap();
            }
        }

        Handled::Ok
    }

    fn on_stop(&mut self) -> Handled {
        global::shutdown_tracer_provider(); // sending remaining spans
        Handled::Ok
    }
}

impl Actor for OrchestraSupervisor {
    type Message = OrchestraSupervisorMessage;

    fn receive_local(&mut self, msg: OrchestraSupervisorMessage) -> Handled {
        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("OrchestraSupervisor")
            .install_simple()
            .unwrap();

        let mut span = match msg {
            OrchestraSupervisorMessage::RegisterProtocolObserver(manifest) => {
                let mut span = tracer.start("register_contracts_observer");
                self.register_protocol_observer(manifest);
                span
            }
            OrchestraSupervisorMessage::ProcessStacksChainEvent(event) => {
                let mut span = tracer.start("handle_stacks_chain_event");
                self.handle_stacks_chain_event(event, &mut span);
                span
            }
            OrchestraSupervisorMessage::ProcessBitcoinChainEvent(event) => {
                let mut span = tracer.start("handle_bitcoin_chain_event");
                self.handle_bitcoin_chain_event(event);
                span
            }
            OrchestraSupervisorMessage::GetProtocolInterfaces(protocol_id, tx) => {
                let mut span = tracer.start("register_local_contracts_observer");
                let worker = match self.active_protocol_observers.get(&protocol_id) {
                    Some(entry) => entry,
                    None => unreachable!(),
                };
                let res = worker.tell(ProtocolObserverMessage::GetInterfaces(tx));
                span
            }
            OrchestraSupervisorMessage::GetFieldValues(request) => {
                let mut span = tracer.start("handle_request_field_value");
                info!(
                    self.ctx.log(),
                    "Contracts observers registered: {:?}", self.active_protocol_observers
                );

                let worker = match self
                    .active_protocol_observers
                    .get(&ProtocolObserverId(request.protocol_id))
                {
                    Some(entry) => entry,
                    None => unreachable!(),
                };
                let res = worker.tell(ProtocolObserverMessage::RequestFieldValues(request));
                span
            }
            OrchestraSupervisorMessage::Exit => {
                let mut span = tracer.start("exit");
                self.ctx.system().shutdown_async();
                span
            }
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}

impl Require<ContractProcessorPort> for OrchestraSupervisor {
    fn handle(&mut self, event: ContractProcessorEvent) -> Handled {
        match event {
            ContractProcessorEvent::TransactionsBatchProcessed(contract_id, events) => {
                let subscriptions = match self.contracts_processors_subscriptions.get(&contract_id)
                {
                    Some(entry) => entry,
                    None => unreachable!(),
                };

                for protocol_id in subscriptions.iter() {
                    info!(self.ctx.log(), "Notifying contract observer");

                    // let worker = match self.active_protocol_observers.get(protocol_id) {
                    //     Some(entry) => entry,
                    //     None => unreachable!(),
                    // };
                    // worker.tell(ProtocolObserverMessage::ProcessChain);
                }
            }
        }
        Handled::Ok
    }
}

impl Require<ProtocolObserverPort> for OrchestraSupervisor {
    fn handle(&mut self, event: ProtocolObserverEvent) -> Handled {
        match event {
            ProtocolObserverEvent::ContractsProcessed(protocol_identifier, full_analysis) => {
                for (contract_id, (analysis, ast, interface, block_identifier)) in full_analysis.into_iter() {
                    if !self.registered_contracts.contains(&contract_id) {
                        self.registered_contracts.insert(contract_id.clone());
                        self.start_contract_processor(contract_id.clone(), interface, analysis, ast, block_identifier);
                    }
                    let worker = match self.active_contracts_processors.get(&contract_id) {
                        Some(worker) => worker,
                        None => unreachable!(),
                    };
        
                    match self
                        .contracts_processors_subscriptions
                        .entry(contract_id)
                    {
                        Entry::Occupied(observers) => {
                            observers.into_mut().insert(protocol_identifier.clone());
                        }
                        Entry::Vacant(entry) => {
                            let mut observers = BTreeSet::new();
                            observers.insert(protocol_identifier.clone());
                            entry.insert(observers);
                        }
                    };
                }
            }
        }
        Handled::Ok
    }
}

impl OrchestraSupervisor {
    pub fn new(storage_driver: StorageDriver) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            contract_processor_port: RequiredPort::uninitialised(),
            protocol_observer_port: RequiredPort::uninitialised(),
            registered_contracts: HashSet::new(),
            bitcoin_predicates: HashMap::new(),
            stacks_predicates: StacksChainPredicates::new(),
            trigger_history: VecDeque::new(),
            storage_driver,
            block_store_manager: None,
            active_contracts_processors: HashMap::new(),
            active_protocol_observers: HashMap::new(),
            contracts_processors_subscriptions: BTreeMap::new(),
        }
    }

    pub fn register_protocol_observer(&mut self, observer_config: ProtocolObserverConfig) {
        let protocol_identifier = &observer_config.identifier;

        if self
            .active_protocol_observers
            .contains_key(&protocol_identifier)
        {
            // todo: or maybe reboot process instead?
            return;
        }

        self.start_protocol_observer(&observer_config);
    }

    pub fn start_contract_processor(&mut self, contract_id: String, interface: ContractInterface, analysis: ContractAnalysis, ast: ContractAST, block_identifier: BlockIdentifier) {
        let system = self.ctx.system();
        let worker = system
            .create(|| ContractProcessor::new(self.storage_driver.clone(), contract_id.clone(), interface, analysis, ast, block_identifier));
        worker.connect_to_required(self.contract_processor_port.share());
        system.start(&worker);
        self.active_contracts_processors
            .insert(contract_id, worker.actor_ref());
    }

    pub fn start_protocol_observer(&mut self, observer_config: &ProtocolObserverConfig) {
        let system = self.ctx.system();
        let worker = system
            .create(|| ProtocolObserver::new(self.storage_driver.clone(), observer_config.clone()));
        worker.connect_to_required(self.protocol_observer_port.share());
        system.start(&worker);
        self.active_protocol_observers
            .insert(observer_config.identifier.clone(), worker.actor_ref());
    }

    pub fn start_block_store_manager(&mut self) {
        let system = self.ctx.system();
        let worker = system.create(|| BlockStoreManager::new(self.storage_driver.clone()));
        system.start(&worker);
        self.block_store_manager = Some(worker.actor_ref());
    }

    pub fn handle_stacks_chain_event(
        &mut self,
        chain_event: StacksChainEvent,
        span: &mut dyn Span,
    ) {
        if self.block_store_manager.is_none() {
            self.start_block_store_manager();
        }

        let worker = match self.block_store_manager {
            Some(ref worker_ref) => worker_ref,
            None => unreachable!(),
        };

        let blocks = match chain_event {
            StacksChainEvent::ChainUpdatedWithBlock(update) => {
                worker.tell(BlockStoreManagerMessage::ArchiveStacksBlock(update.new_block.clone(), update.anchored_trail.clone()));
                vec![(update.new_block.block_identifier, update.new_block.transactions)]
            },
            StacksChainEvent::ChainUpdatedWithReorg(update) => {
                let blocks_ids_to_rollback = update.old_blocks
                    .into_iter()
                    .map(|(old_trail, old_block)| old_block.block_identifier)
                    .collect::<Vec<_>>();

                worker.tell(BlockStoreManagerMessage::RollbackStacksBlocks(
                    blocks_ids_to_rollback,
                ));
                let mut batches = vec![];
                for (anchored_trail, new_block) in update.new_blocks.into_iter() {
                    worker.tell(BlockStoreManagerMessage::ArchiveStacksBlock(new_block.clone(), anchored_trail.clone()));
                    batches.push((new_block.block_identifier, new_block.transactions));
                }
                batches
            }
            StacksChainEvent::ChainUpdatedWithMicroblock(update) => {
                let micro_tip = update.current_trail.microblocks.last().unwrap();
                worker.tell(BlockStoreManagerMessage::ArchiveStacksMicroblock(micro_tip.clone()));
                vec![(micro_tip.block_identifier.clone(), micro_tip.transactions.clone())]
            },
            StacksChainEvent::ChainUpdatedWithMicroblockReorg(_) => {
                unreachable!()
            },
        };

        for (block_identifier, transactions) in blocks.iter() {
            // Send message BlockStoreManagerMessage::ArchiveStacksBlock(block)

            let mut transactions_batches: BTreeMap<&str, Vec<StacksTransactionData>> =
                BTreeMap::new();
            for tx in transactions.iter() {
                let intersect = tx
                    .metadata
                    .receipt
                    .mutated_contracts_radius
                    .intersection(&self.registered_contracts);
                for mutated_contract_id in intersect {
                    match transactions_batches.entry(mutated_contract_id) {
                        Entry::Occupied(transactions) => {
                            transactions.into_mut().push(tx.clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(vec![tx.clone()]);
                        }
                    };
                }
            }

            for (contract_id, batch) in transactions_batches.into_iter() {
                let worker = match self.active_contracts_processors.get(contract_id) {
                    Some(worker) => worker,
                    None => unreachable!(),
                };
                info!(self.log(), "Spawning batch");
                worker.tell(ContractProcessorMessage::ProcessTransactionsBatch(
                    block_identifier.clone(),
                    batch,
                ));
            }
            // todo: keep track of trigger_history.
        }
    }

    pub fn handle_bitcoin_chain_event(&mut self, chain_event: BitcoinChainEvent) {
        if self.block_store_manager.is_none() {
            self.start_block_store_manager();
        }

        let worker = match self.block_store_manager {
            Some(ref worker_ref) => worker_ref,
            None => unreachable!(),
        };

        let blocks = match chain_event {
            BitcoinChainEvent::ChainUpdatedWithBlock(block) => vec![block],
            BitcoinChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {
                let blocks_ids_to_rollback = old_segment
                    .into_iter()
                    .map(|b| b.block_identifier)
                    .collect::<Vec<_>>();

                worker.tell(BlockStoreManagerMessage::RollbackBitcoinBlocks(
                    blocks_ids_to_rollback,
                ));

                // todo: use trigger_history to Rollback previous changes.
                new_segment
            }
        };

        for block in blocks.iter() {
            // Send message BlockStoreManagerMessage::ArchiveStacksBlock(block)
            worker.tell(BlockStoreManagerMessage::ArchiveBitcoinBlock(block.clone()));
        }
    }

    fn handle_new_bitcoin_block(&self, block: BitcoinBlockData) -> HashSet<&TriggerId> {
        let instances_to_trigger: HashSet<&TriggerId> = HashSet::new();
        instances_to_trigger
    }

    pub fn register_predicates(&mut self, mut predicates: StacksChainPredicates) {
        for (k, v) in predicates.watching_contract_id_activity.drain() {
            self.stacks_predicates
                .watching_contract_id_activity
                .insert(k, v);
        }

        for (k, v) in predicates.watching_contract_data_mutation_activity.drain() {
            self.stacks_predicates
                .watching_contract_data_mutation_activity
                .insert(k, v);
        }

        for (k, v) in predicates.watching_principal_activity.drain() {
            self.stacks_predicates
                .watching_principal_activity
                .insert(k, v);
        }

        for (k, v) in predicates.watching_ft_move_activity.drain() {
            self.stacks_predicates
                .watching_ft_move_activity
                .insert(k, v);
        }

        for (k, v) in predicates.watching_nft_activity.drain() {
            self.stacks_predicates.watching_nft_activity.insert(k, v);
        }

        for v in predicates.watching_any_block_activity.drain() {
            self.stacks_predicates.watching_any_block_activity.insert(v);
        }
    }

    pub fn handle_new_stacks_block(
        &self,
        block: StacksBlockData,
        span: &mut dyn Span,
    ) -> HashSet<&TriggerId> {
        let mut instances_to_trigger: HashSet<&TriggerId> = HashSet::new();

        // Start by adding the predicates looking for any new block
        instances_to_trigger.extend(&self.stacks_predicates.watching_any_block_activity);

        for tx in block.transactions.iter() {
            if tx.metadata.success {
                let contract_id_based_predicates = self
                    .evaluate_predicates_watching_contract_mutations_activity(&tx.metadata.receipt);
                instances_to_trigger.extend(&contract_id_based_predicates);
            }
        }

        instances_to_trigger
    }

    fn evaluate_predicates_watching_contract_mutations_activity(
        &self,
        transaction_receipt: &StacksTransactionReceipt,
    ) -> HashSet<&TriggerId> {
        let mut activated_triggers = HashSet::new();

        for contract_id in transaction_receipt.mutated_contracts_radius.iter() {
            if let Some(triggers) = self
                .stacks_predicates
                .watching_contract_id_activity
                .get(contract_id)
            {
                activated_triggers.extend(triggers);
            }
        }

        activated_triggers
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{OrchestraPid, StacksChainPredicates, TriggerId};
    use std::collections::HashSet;

    // #[test]
    // fn test_predicate_watching_contract_id_activity_integration() {

    //     let mut predicates = StacksChainPredicates::new();
    //     let contract_id: String = "STX.contract_id".into();
    //     let mut triggers = HashSet::new();
    //     let trigger_101 = TriggerId { pid: OrchestraPid(1), lambda_id: 1 };
    //     triggers.insert(trigger_101.clone());
    //     predicates.watching_contract_id_activity.insert(contract_id.clone(), triggers);

    //     let mut supervisor = OrchestraSupervisor::new();
    //     supervisor.register_predicates(predicates);

    //     let block = block_with_transactions(vec![
    //         transaction_contract_call_impacting_contract_id(contract_id.clone(), true)
    //     ]);
    //     let res = supervisor.handle_new_stacks_block(block, &mut MockedSpan::new());
    //     assert!(res.contains(&trigger_101));

    //     let block = block_with_transactions(vec![
    //         transaction_contract_call_impacting_contract_id(contract_id.clone(), false)
    //     ]);
    //     let res = supervisor.handle_new_stacks_block(block, &mut MockedSpan::new());
    //     assert!(res.is_empty());
    // }
}
