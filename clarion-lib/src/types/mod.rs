use std::collections::BTreeMap;
use clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;
use clarinet_lib::types::AccountIdentifier;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClarionPid(pub u64);

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
