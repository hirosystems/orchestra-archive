use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{
    ContractInterface, ContractInterfaceAtomType,
};
use clarinet_lib::clarity_repl::clarity::codec::StacksMessageCodec;
use clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;
use clarinet_lib::clarity_repl::clarity::util::bitcoin::blockdata::transaction::Transaction;
use clarinet_lib::clarity_repl::clarity::util::hash::hex_bytes;
use clarinet_lib::clarity_repl::clarity::Value;
use clarinet_lib::types::events::StacksTransactionEvent;
use clarinet_lib::types::{
    AccountIdentifier, BitcoinBlockData, BlockIdentifier, StacksBlockData, TransactionIdentifier,
};
use serde_json::map::Map;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct OrchestraPid(pub u64);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TriggerId {
    pub pid: OrchestraPid,
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
    pub manifest_path: PathBuf,
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
    pub events: Vec<DataVarSetEventFormattedValue>,
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
    pub events: Vec<DataMapEventFormattedValue>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NftValues {
    pub tokens: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    pub tokens_page_size: u16,
    pub tokens_page_index: u64,
    pub token_type: ContractInterfaceAtomType,
    pub events: Vec<NFTEventFormattedValue>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FtValues {
    pub balances: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    pub balances_page_size: u16,
    pub balances_page_index: u64,
    // pub total_supply: Option<String>, ;; TODO: not present in ContractInterface :/
    pub events: Vec<FTEventFormattedValue>,
    pub events_page_size: u16,
    pub events_page_index: u64,
}

#[derive(Clone, Debug)]
pub struct FieldValuesRequest {
    pub tx: Sender<FieldValuesResponse>,
    pub contract_identifier: String,
    pub field_name: String,
    pub protocol_id: u64,
    pub stacks_block_identifier: BlockIdentifier,
}

#[derive(Clone, Debug)]
pub struct FieldValuesResponse {
    pub contract_identifier: String,
    pub field_name: String,
    pub values: FieldValues,
    pub bitcoin_blocks: Vec<BitcoinBlockData>,
    pub stacks_blocks: Vec<StacksBlockData>,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct STXTransferEventValue {
    pub sender: String,
    pub recipient: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct STXMintEventValue {
    pub recipient: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct STXLockEventValue {
    pub locked_amount: String,
    pub unlock_height: String,
    pub locked_address: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct STXBurnEventValue {
    pub sender: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTTransferEventValue {
    pub hex_asset_identifier: String,
    pub sender: String,
    pub recipient: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTMintEventValue {
    pub hex_asset_identifier: String,
    pub recipient: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTBurnEventValue {
    pub hex_asset_identifier: String,
    pub sender: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTTransferEventValue {
    pub sender: String,
    pub recipient: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTMintEventValue {
    pub recipient: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTBurnEventValue {
    pub sender: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DataMapEventStoredValue {
    Insert(DataMapInsertEventValue),
    Update(DataMapUpdateEventValue),
    Delete(DataMapDeleteEventValue),
}

impl DataMapEventStoredValue {
    pub fn get_formatted_decoded_event(
        &self,
        block_index: u64,
        event_index: u64,
    ) -> DataMapEventFormattedValue {
        match &self {
            DataMapEventStoredValue::Insert(data) => {
                let inserted_key = decode_value(&data.hex_inserted_key);
                let inserted_value = decode_value(&data.hex_inserted_value);
                DataMapEventFormattedValue::Insert(DataMapInsertFormattedValue {
                    inserted_key,
                    inserted_value,
                    block_index,
                    event_index,
                })
            }
            DataMapEventStoredValue::Update(data) => {
                let key = decode_value(&data.hex_key);
                let updated_value = decode_value(&data.hex_updated_value);
                DataMapEventFormattedValue::Update(DataMapUpdateFormattedValue {
                    key,
                    updated_value,
                    block_index,
                    event_index,
                })
            }
            DataMapEventStoredValue::Delete(data) => {
                let deleted_key = decode_value(&data.hex_deleted_key);
                DataMapEventFormattedValue::Delete(DataMapDeleteFormattedValue {
                    deleted_key,
                    block_index,
                    event_index,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum NFTEventStoredValue {
    Mint(NFTMintEventValue),
    Transfer(NFTTransferEventValue),
    Burn(NFTBurnEventValue),
}

impl NFTEventStoredValue {
    pub fn get_formatted_decoded_event(
        &self,
        block_index: u64,
        event_index: u64,
    ) -> NFTEventFormattedValue {
        match &self {
            NFTEventStoredValue::Mint(data) => {
                let asset_identifier = decode_value(&data.hex_asset_identifier);
                let recipient = data.recipient.to_string();
                NFTEventFormattedValue::Mint(NFTMintEventFormattedValue {
                    asset_identifier,
                    recipient,
                    block_index,
                    event_index,
                })
            }
            NFTEventStoredValue::Transfer(data) => {
                let asset_identifier = decode_value(&data.hex_asset_identifier);
                let recipient = data.recipient.to_string();
                let sender = data.sender.to_string();
                NFTEventFormattedValue::Transfer(NFTTransferEventFormattedValue {
                    asset_identifier,
                    recipient,
                    sender,
                    block_index,
                    event_index,
                })
            }
            NFTEventStoredValue::Burn(data) => {
                let asset_identifier = decode_value(&data.hex_asset_identifier);
                let sender = data.sender.to_string();
                NFTEventFormattedValue::Burn(NFTBurnEventFormattedValue {
                    asset_identifier,
                    sender,
                    block_index,
                    event_index,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum FTEventStoredValue {
    Mint(FTMintEventValue),
    Transfer(FTTransferEventValue),
    Burn(FTBurnEventValue),
}

impl FTEventStoredValue {
    pub fn get_formatted_decoded_event(
        &self,
        block_index: u64,
        event_index: u64,
    ) -> FTEventFormattedValue {
        match &self {
            FTEventStoredValue::Mint(data) => {
                let amount = data.amount.to_string();
                let recipient = data.recipient.to_string();
                FTEventFormattedValue::Mint(FTMintEventFormattedValue {
                    amount,
                    recipient,
                    block_index,
                    event_index,
                })
            }
            FTEventStoredValue::Transfer(data) => {
                let amount = data.amount.to_string();
                let recipient = data.recipient.to_string();
                let sender = data.sender.to_string();
                FTEventFormattedValue::Transfer(FTTransferEventFormattedValue {
                    amount,
                    recipient,
                    sender,
                    block_index,
                    event_index,
                })
            }
            FTEventStoredValue::Burn(data) => {
                let amount = data.amount.to_string();
                let sender = data.sender.to_string();
                FTEventFormattedValue::Burn(FTBurnEventFormattedValue {
                    amount,
                    sender,
                    block_index,
                    event_index,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapInsertEventValue {
    pub hex_inserted_key: String,
    pub hex_inserted_value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapUpdateEventValue {
    pub hex_key: String,
    pub hex_updated_value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapDeleteEventValue {
    pub hex_deleted_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SmartContractEventValue {
    pub topic: String,
    pub hex_value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapStoredEntry {
    pub hex_key: String,
    pub hex_value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTStoredEntry {
    pub hex_asset_identifier: String,
    pub owner: String,
}

impl DataMapStoredEntry {
    pub fn get_formatted_decoded_key(&self) -> String {
        let value = self.hex_key.clone();
        let raw_value = match value.strip_prefix("0x") {
            Some(raw_value) => raw_value,
            _ => panic!(),
        };
        let bytes = hex_bytes(&raw_value).unwrap();

        let decoded_key = match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
            Ok(value) => value,
            Err(_) => Value::none(),
        };
        let formatted_key = match decoded_key {
            Value::Tuple(pairs) => {
                let mut map = Map::new();
                for (key, value) in pairs.data_map.into_iter() {
                    map.insert(key.to_string(), format!("{}", value).into());
                }
                json!(map).to_string()
            }
            _ => format!("{}", decoded_key),
        };
        formatted_key
    }

    pub fn get_formatted_decoded_value(&self) -> String {
        let value = self.hex_value.clone();
        let raw_value = match value.strip_prefix("0x") {
            Some(raw_value) => raw_value,
            _ => panic!(),
        };
        let bytes = hex_bytes(&raw_value).unwrap();

        let decoded_value = match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
            Ok(decoded_value) => decoded_value,
            Err(e) => Value::none(),
        };
        let formatted_value = match decoded_value {
            Value::Tuple(pairs) => {
                let mut map = Map::new();
                for (key, value) in pairs.data_map.into_iter() {
                    map.insert(key.to_string(), format!("{}", value).into());
                }
                json!(map).to_string()
            }
            _ => format!("{}", decoded_value),
        };
        formatted_value
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataVarStoredValue {
    pub hex_value: String,
}

impl DataVarStoredValue {
    pub fn get_decoded_value(&self) -> Value {
        let value = self.hex_value.clone();
        let raw_value = match value.strip_prefix("0x") {
            Some(raw_value) => raw_value,
            _ => panic!(),
        };
        let bytes = hex_bytes(&raw_value).unwrap();
        match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
            Ok(value) => value,
            Err(_) => Value::none(),
        }
    }

    pub fn get_formatted_decoded_value(&self) -> String {
        format!("{}", self.get_decoded_value())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataVarSetEventValue {
    pub hex_value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataVarSetEventFormattedValue {
    pub value: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DataMapEventFormattedValue {
    Insert(DataMapInsertFormattedValue),
    Update(DataMapUpdateFormattedValue),
    Delete(DataMapDeleteFormattedValue),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapInsertFormattedValue {
    pub inserted_key: String,
    pub inserted_value: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapUpdateFormattedValue {
    pub key: String,
    pub updated_value: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataMapDeleteFormattedValue {
    pub deleted_key: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum NFTEventFormattedValue {
    Mint(NFTMintEventFormattedValue),
    Transfer(NFTTransferEventFormattedValue),
    Burn(NFTBurnEventFormattedValue),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTMintEventFormattedValue {
    pub recipient: String,
    pub asset_identifier: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTTransferEventFormattedValue {
    pub sender: String,
    pub recipient: String,
    pub asset_identifier: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NFTBurnEventFormattedValue {
    pub sender: String,
    pub asset_identifier: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum FTEventFormattedValue {
    Mint(FTMintEventFormattedValue),
    Transfer(FTTransferEventFormattedValue),
    Burn(FTBurnEventFormattedValue),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTMintEventFormattedValue {
    pub recipient: String,
    pub amount: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTTransferEventFormattedValue {
    pub sender: String,
    pub recipient: String,
    pub amount: String,
    pub block_index: u64,
    pub event_index: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FTBurnEventFormattedValue {
    pub sender: String,
    pub amount: String,
    pub block_index: u64,
    pub event_index: u64,
}

pub fn decode_value(input: &str) -> String {
    let value = input.to_string();
    let raw_value = match value.strip_prefix("0x") {
        Some(raw_value) => raw_value,
        _ => panic!(),
    };
    let bytes = hex_bytes(&raw_value).unwrap();

    let decoded_value = match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
        Ok(decoded_value) => decoded_value,
        Err(e) => Value::none(),
    };
    let formatted_value = match decoded_value {
        Value::Tuple(pairs) => {
            let mut map = Map::new();
            for (key, value) in pairs.data_map.into_iter() {
                map.insert(key.to_string(), format!("{}", value).into());
            }
            json!(map).to_string()
        }
        _ => format!("{}", decoded_value),
    };
    formatted_value
}
