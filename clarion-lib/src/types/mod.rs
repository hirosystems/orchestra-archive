use std::collections::{BTreeMap, HashSet, HashMap};
use std::sync::mpsc::Sender;
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{ContractInterfaceAtomType, ContractInterface};
use clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;
use clarinet_lib::clarity_repl::clarity::util::bitcoin::blockdata::transaction::Transaction;
use clarinet_lib::types::{AccountIdentifier, BlockIdentifier, TransactionIdentifier};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClarionPid(pub u64);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TriggerId {
    pub pid: ClarionPid,
    pub lambda_id: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
pub struct ProtocolObserverId(pub u64);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ProtocolObserverConfig {
    pub identifier: ProtocolObserverId,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum FieldValues {
    Var(VarValues),
    Map(MapValues),
    Nft(NftValues),
    Ft(FtValues),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VarValues {
    pub value: String,
    pub value_type: ContractInterfaceAtomType,
    pub events: Vec<u8>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MapValues {
    pub entries: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    pub entries_page_size: u16,
    pub entries_page_index: u64,
    pub key_type: ContractInterfaceAtomType,
    pub value_type: ContractInterfaceAtomType,
    pub events: Vec<u8>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NftValues {
    pub tokens: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    pub tokens_page_size: u16,
    pub tokens_page_index: u64,
    pub token_type: ContractInterfaceAtomType,
    pub events: Vec<u8>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FtValues {
    pub balances: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    pub balances_page_size: u16,
    pub balances_page_index: u64,
    // pub total_supply: Option<String>, ;; TODO: not present in ContractInterface :/
    pub events: Vec<u8>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Clone, Debug)]
pub struct FieldValuesRequest {
    pub tx: Sender<FieldValuesResponse>,
    pub contract_identifier: String,
    pub field_name: String,
    pub protocol_id: u64,
}

#[derive(Clone, Debug)]
pub struct FieldValuesResponse {
    pub contract_identifier: String,
    pub field_name: String,
    pub values: FieldValues,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolRegistration {
    pub contracts: Vec<Contract>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub contract_identifier: String,
    pub interface: ContractInterface,
}
