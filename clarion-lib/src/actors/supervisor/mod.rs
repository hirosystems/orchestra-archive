use crate::actors::{ContractProcessor, ProtocolObserver, BlockStoreManager,  BlockStoreManagerMessage, ContractProcessorMessage, ProtocolObserverMessage};
use crate::datastore::StorageDriver;
use crate::types::{ProtocolObserverConfig, ProtocolObserverId, TriggerId, BitcoinPredicate, StacksChainPredicates, FieldValues, FieldValuesRequest, ProtocolRegistration};
use clarinet_lib::types::{StacksTransactionReceipt, StacksBlockData, BitcoinBlockData, BitcoinChainEvent, StacksChainEvent, StacksTransactionData};
use std::collections::{HashMap, HashSet, VecDeque};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
use std::sync::mpsc::Sender;
use kompact::prelude::*;
use rocksdb::DB;

use opentelemetry::{global, trace::Span};
use opentelemetry::trace::{Tracer};

use super::contract_processor::{ContractProcessorEvent, ContractProcessorPort};

#[derive(Clone, Debug)]
pub enum ClarionSupervisorMessage {
    RegisterProtocolObserver(ProtocolObserverConfig),
    GetProtocolInterfaces(ProtocolObserverId, Sender<ProtocolRegistration>),
    ProcessStacksChainEvent(StacksChainEvent),
    ProcessBitcoinChainEvent(BitcoinChainEvent),
    GetFieldValues(FieldValuesRequest),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct ClarionSupervisor {
    ctx: ComponentContext<Self>,
    active_contracts_processors: HashMap<String, ActorRef<ContractProcessorMessage>>,
    active_protocol_observers: HashMap<ProtocolObserverId, ActorRef<ProtocolObserverMessage>>,
    contracts_processors_subscriptions: BTreeMap<String, BTreeSet<ProtocolObserverId>>,
    block_store_manager: Option<ActorRef<BlockStoreManagerMessage>>, // Todo: switch to event instead
    registered_contracts: HashSet<String>,
    storage_driver: StorageDriver,
    stacks_predicates: StacksChainPredicates,
    contract_processor_port: RequiredPort<ContractProcessorPort>,
    bitcoin_predicates: HashMap<BitcoinPredicate, Vec<TriggerId>>,
    trigger_history: VecDeque<(String, HashSet<TriggerId>)>,
}

// ignore_indications!(SetOffset, DynamicManager);
// ignore_indications!(SetScale, DynamicManager);
// ignore_lifecycle!(ClarionSupervisor);

impl ComponentLifecycle for ClarionSupervisor {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ClarionSupervisor starting");

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

impl Actor for ClarionSupervisor {
    type Message = ClarionSupervisorMessage;

    fn receive_local(&mut self, msg: ClarionSupervisorMessage) -> Handled {

         let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ClarionSupervisor")
            .install_simple().unwrap();

        let mut span = match msg {
            ClarionSupervisorMessage::RegisterProtocolObserver(manifest) => {
                let mut span = tracer.start("register_contracts_observer");
                self.register_contracts_observer(manifest);
                span
            }
            ClarionSupervisorMessage::ProcessStacksChainEvent(event) => {
                let mut span = tracer.start("handle_stacks_chain_event");
                self.handle_stacks_chain_event(event, &mut span);
                span
            }
            ClarionSupervisorMessage::ProcessBitcoinChainEvent(event) => {
                let mut span = tracer.start("handle_bitcoin_chain_event");
                self.handle_bitcoin_chain_event(event);
                span
            }
            ClarionSupervisorMessage::GetProtocolInterfaces(protocol_id, tx) => {
                let mut span = tracer.start("register_local_contracts_observer");
                let worker = match self.active_protocol_observers.get(&protocol_id) {
                    Some(entry) => entry,
                    None => unreachable!(),
                };
                let res = worker.tell(ProtocolObserverMessage::GetInterfaces(tx));
                span
            }
            ClarionSupervisorMessage::GetFieldValues(request) => {
                let mut span = tracer.start("handle_request_field_value");
                info!(self.ctx.log(), "Contracts observers registered: {:?}", self.active_protocol_observers);

                let worker = match self.active_protocol_observers.get(&ProtocolObserverId(request.protocol_id)) {
                    Some(entry) => entry,
                    None => unreachable!(),
                };
                let res = worker.tell(ProtocolObserverMessage::RequestFieldValues(request));
                span
            }
            ClarionSupervisorMessage::Exit => {
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

impl Require<ContractProcessorPort> for ClarionSupervisor {

    fn handle(&mut self, event: ContractProcessorEvent) -> Handled {
        match event {
            ContractProcessorEvent::TransactionsBatchProcessed(contract_id, events) => {

                let subscriptions = match self.contracts_processors_subscriptions.get(&contract_id) {
                    Some(entry) => entry,
                    None => unreachable!()
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

impl ClarionSupervisor {
    pub fn new(storage_driver: StorageDriver) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            contract_processor_port: RequiredPort::uninitialised(),
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

    pub fn register_contracts_observer(&mut self, observer_config: ProtocolObserverConfig) {

        let protocol_identifier = &observer_config.identifier;

        if self.active_protocol_observers.contains_key(&protocol_identifier) {
            // todo: or maybe reboot process instead?
            return
        } else {
            self.start_contracts_observer(&observer_config);
        }

        for (contract_id, settings) in observer_config.contracts.iter() {
            let contract_id_ser = contract_id.to_string();
            if !self.registered_contracts.contains(&contract_id_ser) {
                self.registered_contracts.insert(contract_id_ser.clone());
                self.start_contract_processor(contract_id_ser.clone());
            }
            let worker = match self.active_contracts_processors.get(&contract_id_ser) {
                Some(worker) => worker,
                None => unreachable!()
            };

            // todo: boot worker?

            match self.contracts_processors_subscriptions.entry(contract_id_ser) {
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

    pub fn start_contract_processor(&mut self, contract_id: String) {
        let system = self.ctx.system();
        let worker = system.create(|| ContractProcessor::new(self.storage_driver.clone(), contract_id.clone()));
        worker.connect_to_required(self.contract_processor_port.share());
        system.start(&worker);
        self.active_contracts_processors.insert(contract_id, worker.actor_ref());
    }

    pub fn start_contracts_observer(&mut self, observer_config: &ProtocolObserverConfig) {
        let system = self.ctx.system();
        let worker = system.create(|| ProtocolObserver::new(self.storage_driver.clone(), observer_config.clone()));
        system.start(&worker);
        self.active_protocol_observers.insert(observer_config.identifier.clone(), worker.actor_ref());
    }

    pub fn start_block_store_manager(&mut self) {
        let system = self.ctx.system();
        let worker = system.create(|| BlockStoreManager::new(self.storage_driver.clone()));
        system.start(&worker);
        self.block_store_manager = Some(worker.actor_ref());
    }

    pub fn handle_stacks_chain_event(&mut self, chain_event: StacksChainEvent, span: &mut dyn Span) {

        if self.block_store_manager.is_none() {
            self.start_block_store_manager();
        }

        let worker = match self.block_store_manager {
            Some(ref worker_ref) => worker_ref,
            None => unreachable!()
        };

        let blocks = match chain_event {
            StacksChainEvent::ChainUpdatedWithBlock(block) => vec![block],
            StacksChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {
                let blocks_ids_to_rollback = old_segment
                    .into_iter()
                    .map(|b| b.block_identifier)
                    .collect::<Vec<_>>();

                worker.tell(BlockStoreManagerMessage::RollbackStacksBlocks(blocks_ids_to_rollback));

                // todo: use trigger_history to Rollback previous changes.
                new_segment
            }
        };

        for block in blocks.iter() {
            // Send message BlockStoreManagerMessage::ArchiveStacksBlock(block)
            worker.tell(BlockStoreManagerMessage::ArchiveStacksBlock(block.clone()));

            let mut transactions_batches: BTreeMap<&str, Vec<StacksTransactionData>> = BTreeMap::new();
            for tx in block.transactions.iter() {
                let intersect = tx.metadata.receipt.mutated_contracts_radius.intersection(&self.registered_contracts);
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
                    None => unreachable!()
                };
                info!(self.log(), "Spawning batch");
                worker.tell(ContractProcessorMessage::ProcessTransactionsBatch(block.block_identifier.clone(), batch));
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
            None => unreachable!()
        };

        let blocks = match chain_event {
            BitcoinChainEvent::ChainUpdatedWithBlock(block) => vec![block],
            BitcoinChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {
                let blocks_ids_to_rollback = old_segment
                    .into_iter()
                    .map(|b| b.block_identifier)
                    .collect::<Vec<_>>();

                worker.tell(BlockStoreManagerMessage::RollbackBitcoinBlocks(blocks_ids_to_rollback));

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
            self.stacks_predicates.watching_contract_id_activity.insert(k, v);
        }

        for (k, v) in predicates.watching_contract_data_mutation_activity.drain() {
            self.stacks_predicates.watching_contract_data_mutation_activity.insert(k, v);
        }

        for (k, v) in predicates.watching_principal_activity.drain() {
            self.stacks_predicates.watching_principal_activity.insert(k, v);
        }

        for (k, v) in predicates.watching_ft_move_activity.drain() {
            self.stacks_predicates.watching_ft_move_activity.insert(k, v);
        }

        for (k, v) in predicates.watching_nft_activity.drain() {
            self.stacks_predicates.watching_nft_activity.insert(k, v);
        }

        for v in predicates.watching_any_block_activity.drain() {
            self.stacks_predicates.watching_any_block_activity.insert(v);
        }
    }

    pub fn handle_new_stacks_block(&self, block: StacksBlockData, span: &mut dyn Span) -> HashSet<&TriggerId> {
        let mut instances_to_trigger: HashSet<&TriggerId> = HashSet::new();

        // Start by adding the predicates looking for any new block
        instances_to_trigger.extend(&self.stacks_predicates.watching_any_block_activity);

        for tx in block.transactions.iter() {
            if tx.metadata.success {
                let contract_id_based_predicates = self
                    .evaluate_predicates_watching_contract_mutations_activity(
                        &tx.metadata.receipt,
                    );
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
    use crate::types::{ClarionPid, StacksChainPredicates, TriggerId};
    use std::collections::HashSet;
    
    // #[test]
    // fn test_predicate_watching_contract_id_activity_integration() {

    //     let mut predicates = StacksChainPredicates::new();
    //     let contract_id: String = "STX.contract_id".into();
    //     let mut triggers = HashSet::new();
    //     let trigger_101 = TriggerId { pid: ClarionPid(1), lambda_id: 1 };
    //     triggers.insert(trigger_101.clone());
    //     predicates.watching_contract_id_activity.insert(contract_id.clone(), triggers);

    //     let mut supervisor = ClarionSupervisor::new();
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
