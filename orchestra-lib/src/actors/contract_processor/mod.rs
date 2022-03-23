use crate::datastore::blocks::{self, stacks_blocks_db_read};
use crate::datastore::contracts::{self, contract_db_delete_all, contract_db_write, db_key};
use crate::types::{
    DataMapDeleteEventValue, DataMapEventStoredValue, DataMapInsertEventValue, DataMapStoredEntry,
    DataMapUpdateEventValue, DataVarSetEventValue, DataVarStoredValue, FTBurnEventValue,
    FTEventStoredValue, FTMintEventValue, FTTransferEventValue, NFTBurnEventValue,
    NFTEventStoredValue, NFTMintEventValue, NFTStoredEntry, NFTTransferEventValue,
    SmartContractEventValue,
};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::build_contract_interface;
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{
    ContractInterface, ContractInterfaceAtomType,
};
use clarinet_lib::clarity_repl::clarity::analysis::ContractAnalysis;
use clarinet_lib::clarity_repl::clarity::types::{
    QualifiedContractIdentifier, StandardPrincipalData,
};
use clarinet_lib::clarity_repl::clarity::util::hash::{hex_bytes, to_hex};
use clarinet_lib::clarity_repl::repl::ast::ContractAST;
use clarinet_lib::clarity_repl::repl::settings::InitialContract;
use clarinet_lib::clarity_repl::repl::{ClarityInterpreter, Session, SessionSettings};
use clarinet_lib::types::events::{SmartContractEventData, StacksTransactionEvent};
use clarinet_lib::types::{
    BlockIdentifier, StacksBlockData, StacksTransactionData, TransactionIdentifier,
};
use kompact::prelude::*;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use rocksdb::{Options, DB};

use std::collections::{BTreeMap, VecDeque};

use crate::datastore::StorageDriver;

use super::block_store_manager::ContractInstanciation;

#[derive(Clone, Debug)]
pub enum ContractProcessorMessage {
    RebuildState,
    ProcessTransactionsBatch(BlockIdentifier, Vec<StacksTransactionData>),
    RollbackTransactionsBatch(BlockIdentifier, Vec<StacksTransactionData>),
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractProcessorEvent {
    TransactionsBatchProcessed(String, Vec<(TransactionIdentifier, SmartContractEventData)>),
}

pub struct ContractProcessorPort;

impl Port for ContractProcessorPort {
    type Indication = ContractProcessorEvent;
    type Request = Never;
}

#[derive(ComponentDefinition)]
pub struct ContractProcessor {
    ctx: ComponentContext<Self>,
    contract_processor_port: ProvidedPort<ContractProcessorPort>,
    contract_id: String,
    contract_interface: ContractInterface,
    storage_driver: StorageDriver,
    block_identifier: BlockIdentifier,
    analysis: ContractAnalysis,
    ast: ContractAST,
}
pub enum Changes<'a> {
    UpdateDataVar(&'a str, &'a str, &'a str),
    InsertDataMapEntry(&'a str, (&'a str, &'a str), &'a str),
    DeleteDataMapEntry(&'a str, &'a str, &'a str),
    UpdateDataMapEntry(&'a str, (&'a str, &'a str), &'a str),
    SendTokens(&'a str, (&'a str, u128), &'a str),
    ReceiveTokens(&'a str, (&'a str, u128), &'a str),
    SendNFT(&'a str, (&'a str, &'a str), &'a str),
    ReceiveNFT(&'a str, (&'a str, &'a str), &'a str),
}

ignore_requests!(ContractProcessorPort, ContractProcessor);

impl ContractProcessor {
    pub fn new(
        storage_driver: StorageDriver,
        contract_id: String,
        contract_interface: ContractInterface,
        analysis: ContractAnalysis,
        ast: ContractAST,
        block_identifier: BlockIdentifier,
    ) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            contract_processor_port: ProvidedPort::uninitialised(),
            storage_driver,
            contract_id,
            contract_interface,
            block_identifier,
            analysis,
            ast,
        }
    }

    fn db_key(&self, key: contracts::DBKey) -> Vec<u8> {
        db_key(key, &self.contract_id)
    }

    pub fn build_state(&mut self) {
        {
            contract_db_delete_all(&self.storage_driver, &self.contract_id);
            let db = contract_db_write(&self.storage_driver, &self.contract_id);
            let interface = build_contract_interface(&self.analysis);
            let interface_bytes =
                serde_json::to_vec(&interface).expect("Unable to serialize block");
            db.put(&self.db_key(contracts::DBKey::Interface), interface_bytes)
                .unwrap();
        }
        let block_db = stacks_blocks_db_read(&self.storage_driver);
        let start = self.block_identifier.index;
        let end = u64::from_be_bytes(
            block_db
                .get("tip".as_bytes())
                .unwrap()
                .unwrap()
                .try_into()
                .unwrap(),
        );

        warn!(
            self.ctx().log(),
            "Will be looking for stacks blocks in range {:?}",
            start..=end
        );
        for index in start..=end {
            let block_hash = block_db.get(index.to_be_bytes()).unwrap().unwrap();
            let key = format!("hash:{}", String::from_utf8(block_hash).unwrap());
            warn!(self.ctx().log(), "Getting {}", key);

            let block_bytes = block_db.get(&key.as_bytes()).unwrap().unwrap();
            let block = serde_json::from_slice::<StacksBlockData>(&block_bytes)
                .expect("Unable to deserialize contract");

            let mut transactions = vec![];
            for transaction in block.transactions.iter() {
                if transaction
                    .metadata
                    .receipt
                    .mutated_contracts_radius
                    .contains(&self.contract_id)
                {
                    transactions.push(transaction.clone());
                }
            }
            if !transactions.is_empty() {
                self.handle_transactions_batch(block.block_identifier.clone(), transactions);
            }
        }
    }

    fn handle_transactions_batch(
        &mut self,
        block_identifier: BlockIdentifier,
        transactions: Vec<StacksTransactionData>,
    ) -> Vec<(TransactionIdentifier, SmartContractEventData)> {
        let mut changes = vec![];
        let mut custom_events = vec![];
        let mut event_index = 0;
        let db = contract_db_write(&self.storage_driver, &self.contract_id);
        for tx in transactions.iter() {
            for event_wrapper in tx.metadata.receipt.events.iter() {
                match event_wrapper {
                    StacksTransactionEvent::DataVarSetEvent(event) => {
                        if event.contract_identifier == self.contract_id {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::VarEvent(
                                    &event.var,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(DataVarSetEventValue {
                                    hex_value: event.hex_new_value.to_string(),
                                })
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::UpdateDataVar(
                                &event.var,
                                &event.hex_new_value,
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::DataMapInsertEvent(event) => {
                        if event.contract_identifier == self.contract_id {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::MapEvent(
                                    &event.map,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(DataMapEventStoredValue::Insert(DataMapInsertEventValue {
                                    hex_inserted_key: event.hex_inserted_key.to_string(),
                                    hex_inserted_value: event.hex_inserted_value.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::InsertDataMapEntry(
                                &event.map,
                                (&event.hex_inserted_key, &event.hex_inserted_value),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::DataMapUpdateEvent(event) => {
                        if event.contract_identifier == self.contract_id {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::MapEvent(
                                    &event.map,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(DataMapEventStoredValue::Update(DataMapUpdateEventValue {
                                    hex_key: event.hex_key.to_string(),
                                    hex_updated_value: event.hex_new_value.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::UpdateDataMapEntry(
                                &event.map,
                                (&event.hex_key, &event.hex_new_value),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::DataMapDeleteEvent(event) => {
                        if event.contract_identifier == self.contract_id {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::MapEvent(
                                    &event.map,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(DataMapEventStoredValue::Delete(DataMapDeleteEventValue {
                                    hex_deleted_key: event.hex_deleted_key.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::DeleteDataMapEntry(
                                &event.map,
                                &event.hex_deleted_key,
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::FTMintEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::FTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(FTEventStoredValue::Mint(FTMintEventValue {
                                    recipient: event.recipient.to_string(),
                                    amount: event.amount.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            let amount = u128::from_str_radix(&event.amount, 10)
                                .expect("unable to parse amount");
                            changes.push(Changes::ReceiveTokens(
                                &event.asset_class_identifier,
                                (&event.recipient, amount),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::FTBurnEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::FTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(FTEventStoredValue::Burn(FTBurnEventValue {
                                    sender: event.sender.to_string(),
                                    amount: event.amount.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            let amount = u128::from_str_radix(&event.amount, 10)
                                .expect("unable to parse amount");
                            changes.push(Changes::SendTokens(
                                &event.asset_class_identifier,
                                (&event.sender, amount),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::FTTransferEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::FTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(FTEventStoredValue::Transfer(FTTransferEventValue {
                                    sender: event.sender.to_string(),
                                    recipient: event.recipient.to_string(),
                                    amount: event.amount.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            let amount = u128::from_str_radix(&event.amount, 10)
                                .expect("unable to parse amount");
                            changes.push(Changes::SendTokens(
                                &event.asset_class_identifier,
                                (&event.sender, amount),
                                &tx.transaction_identifier.hash,
                            ));
                            changes.push(Changes::ReceiveTokens(
                                &event.asset_class_identifier,
                                (&event.recipient, amount),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::NFTMintEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::NFTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(NFTEventStoredValue::Mint(NFTMintEventValue {
                                    recipient: event.recipient.to_string(),
                                    hex_asset_identifier: event.hex_asset_identifier.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::ReceiveNFT(
                                &event.asset_class_identifier,
                                (&event.hex_asset_identifier, &event.recipient),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::NFTBurnEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::NFTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(NFTEventStoredValue::Burn(NFTBurnEventValue {
                                    sender: event.sender.to_string(),
                                    hex_asset_identifier: event.hex_asset_identifier.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::SendNFT(
                                &event.asset_class_identifier,
                                (&event.hex_asset_identifier, &event.sender),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::NFTTransferEvent(event) => {
                        if event.asset_class_identifier.starts_with(&self.contract_id) {
                            event_index += 1;
                            db.put(
                                &self.db_key(contracts::DBKey::NFTEvent(
                                    &event.asset_class_identifier,
                                    block_identifier.index,
                                    event_index,
                                )),
                                json!(NFTEventStoredValue::Transfer(NFTTransferEventValue {
                                    sender: event.sender.to_string(),
                                    recipient: event.recipient.to_string(),
                                    hex_asset_identifier: event.hex_asset_identifier.to_string(),
                                }))
                                .to_string()
                                .as_bytes(),
                            )
                            .expect("Unable to write");
                            changes.push(Changes::SendNFT(
                                &event.asset_class_identifier,
                                (&event.hex_asset_identifier, &event.sender),
                                &tx.transaction_identifier.hash,
                            ));
                            changes.push(Changes::ReceiveNFT(
                                &event.asset_class_identifier,
                                (&event.hex_asset_identifier, &event.recipient),
                                &tx.transaction_identifier.hash,
                            ))
                        }
                    }
                    StacksTransactionEvent::SmartContractEvent(event) => {
                        if event.contract_identifier == self.contract_id {
                            custom_events.push((tx.transaction_identifier.clone(), event.clone()));
                        }
                    }
                    StacksTransactionEvent::STXMintEvent(event) => {}
                    StacksTransactionEvent::STXBurnEvent(event) => {}
                    StacksTransactionEvent::STXTransferEvent(event) => {}
                    StacksTransactionEvent::STXLockEvent(event) => {}
                }
            }
        }

        {
            for change in changes.iter() {
                match change {
                    Changes::UpdateDataVar(var, new_value, txid) => {
                        db.put(
                            &self.db_key(contracts::DBKey::Var(var)),
                            json!(DataVarStoredValue {
                                hex_value: new_value.to_string(),
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .expect("Unable to write");
                    }
                    Changes::InsertDataMapEntry(map, (new_key, new_value), txid) => {
                        db.put(
                            &self.db_key(contracts::DBKey::MapEntry(map, new_key)),
                            json!(DataMapStoredEntry {
                                hex_key: new_key.to_string(),
                                hex_value: new_value.to_string()
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .expect("Unable to write");
                    }
                    Changes::DeleteDataMapEntry(map, deleted_key, txid) => {
                        db.delete(&self.db_key(contracts::DBKey::MapEntry(map, deleted_key)))
                            .expect("Unable to write");
                    }
                    Changes::UpdateDataMapEntry(map, (key, new_value), txid) => {
                        db.put(
                            &self.db_key(contracts::DBKey::MapEntry(map, key)),
                            json!(DataMapStoredEntry {
                                hex_key: key.to_string(),
                                hex_value: new_value.to_string()
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .expect("Unable to write");
                    }
                    Changes::SendTokens(asset_id, (sender, value), txid) => {
                        let balance =
                            match db.get(&self.db_key(contracts::DBKey::FT(asset_id, sender))) {
                                Ok(Some(value)) => {
                                    u128::from_str_radix(&String::from_utf8(value).unwrap(), 10)
                                        .unwrap()
                                }
                                Ok(None) => 0,
                                Err(e) => panic!("Operational problem encountered: {}", e),
                            };
                        info!(
                            self.log(),
                            "{} will send {} (balance={})", sender, value, balance
                        );

                        db.put(
                            &self.db_key(contracts::DBKey::FT(asset_id, sender)),
                            (balance - value).to_string(),
                        )
                        .expect("Unable to write");
                    }
                    Changes::ReceiveTokens(asset_id, (recipient, value), txid) => {
                        let balance =
                            match db.get(&self.db_key(contracts::DBKey::FT(asset_id, recipient))) {
                                Ok(Some(value)) => {
                                    u128::from_str_radix(&String::from_utf8(value).unwrap(), 10)
                                        .unwrap()
                                }
                                Ok(None) => 0,
                                Err(e) => panic!("Operational problem encountered: {}", e),
                            };
                        info!(
                            self.log(),
                            "{} will receive {} (balance={})", recipient, value, balance
                        );

                        db.put(
                            &self.db_key(contracts::DBKey::FT(asset_id, recipient)),
                            (balance + value).to_string(),
                        )
                        .expect("Unable to write");
                    }
                    Changes::SendNFT(asset_class_id, (asset_id, sender), txid) => {
                        db.delete(&self.db_key(contracts::DBKey::NFT(asset_class_id, asset_id)))
                            .expect("Unable to write");
                    }
                    Changes::ReceiveNFT(asset_class_id, (asset_id, recipient), txid) => {
                        db.put(
                            &self.db_key(contracts::DBKey::NFT(asset_class_id, asset_id)),
                            json!(NFTStoredEntry {
                                hex_asset_identifier: asset_id.to_string(),
                                owner: recipient.to_string(),
                            })
                            .to_string()
                            .as_bytes(),
                        )
                        .expect("Unable to write");
                    }
                }
            }
        }
        custom_events
    }
}

impl ComponentLifecycle for ContractProcessor {
    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ContractProcessor starting and building state");
        self.build_state();
        Handled::Ok
    }
}

impl Actor for ContractProcessor {
    type Message = ContractProcessorMessage;

    fn receive_local(&mut self, msg: ContractProcessorMessage) -> Handled {
        info!(self.ctx.log(), "ContractProcessor received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ContractProcessor")
            .install_simple()
            .unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ContractProcessorMessage::RebuildState => {}
            ContractProcessorMessage::ProcessTransactionsBatch(block_identifier, transactions) => {
                info!(
                    self.ctx.log(),
                    "ContractProcessor processed transaction batch"
                );

                let custom_events = self.handle_transactions_batch(block_identifier, transactions);

                self.contract_processor_port.trigger(
                    ContractProcessorEvent::TransactionsBatchProcessed(
                        self.contract_id.clone(),
                        custom_events,
                    ),
                )
            }
            ContractProcessorMessage::RollbackTransactionsBatch(block_identifier, transactions) => {
            }
            ContractProcessorMessage::Exit => {}
        };
        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
