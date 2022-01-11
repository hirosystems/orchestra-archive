use crate::actors::contract_processor::ContractProcessor;
use clarinet_lib::types::{AccountIdentifier, StacksTransactionReceipt, StacksBlockData, BitcoinBlockData, BitcoinChainEvent, StacksChainEvent};
use clarinet_lib::clarity_repl::clarity::types::{QualifiedContractIdentifier};
use kompact::lookup::ActorLookup;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::sync::mpsc::Sender;
use kompact::{component::AbstractComponent, prelude::*};
use std::sync::Arc;


use opentelemetry::{global, trace::Span};
use opentelemetry::trace::{Tracer, SpanContext};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TriggerId {
    pub pid: ClarionPid,
    pub lambda_id: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClarionManifest {
    pub project: ProjectMetadata,
    pub lambdas: Vec<Lambda>,
    pub contracts: BTreeMap<QualifiedContractIdentifier, ContractSettings>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ProjectMetadata {
    pub name: String,
    pub authors: Vec<String>,
    pub homepage: String,
    pub license: String,
    pub description: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ContractSettings {
    pub state_explorer_enabled: bool,
    pub api_generator_enabled: Vec<String>,
}

pub enum ClarionInstanceCommand {
    Start,
    Stop,
    AddLambda,
}

#[derive(Debug)]
pub struct ClarionInstanceController {
    pid: ClarionPid,
    tx: Sender<ClarionInstanceCommand>,
}

impl ClarionInstanceController {
    pub fn trigger_lambda(&self, lambda_id: u64) {
        println!("Triggering lambda {}", lambda_id);
    }
}

#[derive(Clone, Debug)]
pub enum ClarionSupervisorMessage {
    RegisterManifest(ClarionManifest),
    ProcessStacksChainEvent(StacksChainEvent),
    ProcessBitcoinChainEvent(BitcoinChainEvent),
    Exit,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClarionPid(u64);

#[derive(ComponentDefinition)]
pub struct ClarionSupervisor {
    ctx: ComponentContext<Self>,
    units: Vec<Arc<dyn AbstractComponent<Message = f32>>>,
    instances_pool: HashSet<ClarionPid>,
    clarion_controllers: HashMap<ClarionPid, ClarionInstanceController>,
    bitcoin_predicates: HashMap<BitcoinPredicate, Vec<TriggerId>>,
    stacks_predicates: StacksChainPredicates,
    registered_contracts: HashSet<QualifiedContractIdentifier>,
    registered_manifests: HashSet<ClarionManifest>,
    trigger_history: VecDeque<(String, HashSet<TriggerId>)>,
}

// ignore_indications!(SetOffset, DynamicManager);
// ignore_indications!(SetScale, DynamicManager);
// ignore_lifecycle!(ClarionSupervisor);

impl ComponentLifecycle for ClarionSupervisor {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ClarionSupervisor starting");
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

        // let tracer = global::tracer("ClarionSupervisor");

        let mut span = match msg {
            ClarionSupervisorMessage::RegisterManifest(manifest) => {
                let mut span = tracer.start("spawn contract processor");
                self.register_manifest(manifest);
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
            ClarionSupervisorMessage::Exit => {
                let mut span = tracer.start("handle_shutdown");
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

impl ClarionSupervisor {
    pub fn new() -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            units: vec![],    
            instances_pool: HashSet::new(),
            registered_contracts: HashSet::new(),
            registered_manifests: HashSet::new(),
            clarion_controllers: HashMap::new(),
            bitcoin_predicates: HashMap::new(),
            stacks_predicates: StacksChainPredicates::new(),
            trigger_history: VecDeque::new(),
        }
    }

    pub fn register_manifest(&mut self, manifest: ClarionManifest) {

        if self.registered_manifests.contains(&manifest) {
            return
        }

        for (contract_id, settings) in manifest.contracts.iter() {
            if self.registered_contracts.contains(contract_id) {
                self.start_contract_processor_observer(contract_id, &manifest);
            } else {
                self.registered_contracts.insert(contract_id.clone());
                self.start_contract_processor(contract_id);
            }
        } 
    }

    pub fn start_contract_processor(&mut self, contract_id: &QualifiedContractIdentifier) {
        let system = self.ctx.system();
        let instance = system.create(ContractProcessor::new);
        system.start(&instance);
        // self.clarion_controllers.insert(pid.clone(), controller);
        // self.instances_pool.insert(pid, instance);
    }

    pub fn start_contract_processor_observer(&mut self, contract_id: &QualifiedContractIdentifier, manifest: &ClarionManifest) {
        let system = self.ctx.system();
        let instance = system.create(ContractProcessor::new);
        system.start(&instance);
        // self.clarion_controllers.insert(pid.clone(), controller);
        // self.instances_pool.insert(pid, instance);
    }

    pub fn handle_stacks_chain_event(&mut self, chain_event: StacksChainEvent, span: &mut dyn Span) {
        match chain_event {
            StacksChainEvent::ChainUpdatedWithBlock(new_block) => {
                let jobs = self.handle_new_stacks_block(new_block, span);
                // todo: keep track of trigger_history.
            }
            StacksChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {
                // TODO(lgalabru): handle
                // todo: keep track of trigger_history.
            }
        }
    }

    pub fn handle_bitcoin_chain_event(&mut self, chain_event: BitcoinChainEvent) {
        match chain_event {
            BitcoinChainEvent::ChainUpdatedWithBlock(new_block) => {
                let jobs = self.handle_new_bitcoin_block(new_block);
            }
            BitcoinChainEvent::ChainUpdatedWithReorg(old_segment, new_segment) => {
                // TODO(lgalabru): handle
            }
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

        for trigger in instances_to_trigger.iter() {
            if let Some(controller) = self.clarion_controllers.get(&trigger.pid) {
                controller.trigger_lambda(trigger.lambda_id);
            }
        }
        instances_to_trigger
    }

    fn evaluate_predicates_watching_contract_mutations_activity(
        &self,
        transaction_receipt: &StacksTransactionReceipt,
    ) -> HashSet<&TriggerId> {
        let mut activated_triggers = HashSet::new();

        
        for contract_id in transaction_receipt.contracts_execution_radius.iter() {
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

pub struct StacksChainPredicates {
    pub watching_contract_id_activity: HashMap<String, HashSet<TriggerId>>,
    pub watching_contract_data_mutation_activity: HashMap<String, HashSet<TriggerId>>,
    pub watching_principal_activity: HashMap<String, HashSet<TriggerId>>,
    pub watching_ft_move_activity: HashMap<String, HashSet<TriggerId>>,
    pub watching_nft_activity: HashMap<String, HashSet<TriggerId>>,
    pub watching_any_block_activity: HashSet<TriggerId>,
}

impl StacksChainPredicates {
    pub fn new() -> Self {
        Self {
            watching_contract_id_activity: HashMap::new(),
            watching_contract_data_mutation_activity: HashMap::new(),
            watching_principal_activity: HashMap::new(),
            watching_ft_move_activity: HashMap::new(),
            watching_nft_activity: HashMap::new(),
            watching_any_block_activity: HashSet::new(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Lambda {
    lambda_id: u64,
    name: String,
    predicate: Predicate,
    action: Action,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Action {
    User,
    Platform,
}

pub enum User {
    HTTPPost(String),
    CodeExecution(String),
}

pub enum Platform {
    StateExplorer,
    ApiGenerator,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Predicate {
    BitcoinPredicate,
    StacksPredicate,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum BitcoinPredicate {
    AnyBlock,
    AnyOperation(AccountIdentifier),
    AnyStacksOperation(CrossStacksChainOperation, AccountIdentifier),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum CrossStacksChainOperation {
    Any,
    MineBlock,
    TransferSTX,
    StacksSTX,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum StacksPredicate {
    BitcoinPredicate,
    StacksContractPredicate,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum StacksContractBasedPredicate {
    AnyCallToContract(QualifiedContractIdentifier),
    AnyResultFromContractCall(QualifiedContractIdentifier, String),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum StacksOperationPredicate {
    AnyOperation(AccountIdentifier),
}

use std::time::SystemTime;
use opentelemetry::{KeyValue};
use opentelemetry::trace::{StatusCode};
use clarinet_lib::types::{BlockIdentifier, StacksBlockMetadata, StacksTransactionData, TransactionIdentifier, StacksTransactionMetadata};

#[derive(Debug)]
struct MockedSpan {
    context: SpanContext
}

impl MockedSpan {
    pub fn new() -> MockedSpan {
        MockedSpan {
            context: SpanContext::empty_context(),
        }
    }
}

impl Span for MockedSpan {
    fn add_event_with_timestamp(
        &mut self,
        _name: String,
        _timestamp: SystemTime,
        _attributes: Vec<KeyValue>,
    ) {}
    fn span_context(&self) -> &SpanContext {
        return &self.context
    }
    fn is_recording(&self) -> bool { true }
    fn set_attribute(&mut self, _attribute: KeyValue) {}
    fn set_status(&mut self, _code: StatusCode, _message: String) {}
    fn update_name(&mut self, _new_name: String) {}
    fn end(&mut self) {}
    fn end_with_timestamp(&mut self, _timestamp: SystemTime) {}
}


fn transaction_impacting_contract_id(contract_id: String, success: bool) -> StacksTransactionData {
    let mut contracts_execution_radius = HashSet::new();
    contracts_execution_radius.insert(contract_id);
    StacksTransactionData {
        transaction_identifier: TransactionIdentifier {
            hash: "0".into()
        },
        operations: vec![],
        metadata: StacksTransactionMetadata {
            success,
            result: "".into(),
            receipt: StacksTransactionReceipt {
                contracts_execution_radius,
                assets_mutation_radius: HashSet::new(),
                events: vec![],
            },
            description: "".into(),
        }
    }
}

fn block_with_transactions(transactions: Vec<StacksTransactionData>) -> StacksBlockData {
    StacksBlockData {
        block_identifier: BlockIdentifier { index: 1, hash: "1".into() },
        parent_block_identifier: BlockIdentifier { index: 0, hash: "0".into() },
        timestamp: 0,
        transactions,
        metadata: StacksBlockMetadata { 
            bitcoin_anchor_block_identifier: BlockIdentifier { index: 0, hash: "0".into() }, 
            pox_cycle_index: 0, 
            pox_cycle_position: 0,
            pox_cycle_length: 0 
        }
    }
}


#[test]
fn test_predicate_watching_contract_id_activity_integration() {

    let mut predicates = StacksChainPredicates::new();
    let contract_id: String = "STX.contract_id".into();
    let mut triggers = HashSet::new();
    let trigger_101 = TriggerId { pid: ClarionPid(1), lambda_id: 1 };
    triggers.insert(trigger_101.clone());
    predicates.watching_contract_id_activity.insert(contract_id.clone(), triggers);

    let mut supervisor = ClarionSupervisor::new();
    supervisor.register_predicates(predicates);

    let block = block_with_transactions(vec![
        transaction_impacting_contract_id(contract_id.clone(), true)
    ]);
    let res = supervisor.handle_new_stacks_block(block, &mut MockedSpan::new());
    assert!(res.contains(&trigger_101));

    let block = block_with_transactions(vec![
        transaction_impacting_contract_id(contract_id.clone(), false)
    ]);
    let res = supervisor.handle_new_stacks_block(block, &mut MockedSpan::new());
    assert!(res.is_empty());
}

