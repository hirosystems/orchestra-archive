use crate::types::ProtocolObserverConfig;
use clarinet_lib::clarity_repl::clarity::types::{StandardPrincipalData, QualifiedContractIdentifier};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::build_contract_interface;
use clarinet_lib::clarity_repl::clarity::util::hash::hex_bytes;
use clarinet_lib::clarity_repl::repl::settings::InitialContract;
use clarinet_lib::clarity_repl::repl::{ClarityInterpreter, Session, SessionSettings};
use clarinet_lib::types::{StacksTransactionData, TransactionIdentifier};
use clarinet_lib::types::events::{StacksTransactionEvent, SmartContractEventData};
use kompact::prelude::*;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use rocksdb::{Options, DB};
use crate::datastore::contracts::{db_key, DBKey, contract_db_read, contract_db_write};

use std::collections::{BTreeMap, VecDeque};

use crate::datastore::StorageDriver;

use super::block_store_manager::ContractInstanciation;

#[derive(Clone, Debug)]
pub enum ContractProcessorMessage {
    RebuildState,
    ProcessTransactionsBatch(Vec<StacksTransactionData>),
    RollbackTransactionsBatch(Vec<StacksTransactionData>),
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractProcessorEvent {
    TransactionsBatchProcessed(String, Vec<(SmartContractEventData, TransactionIdentifier)>),
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
    storage_driver: StorageDriver,
}
pub enum Changes <'a> {
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
    pub fn new(storage_driver: StorageDriver, contract_id: String) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            contract_processor_port: ProvidedPort::uninitialised(),
            storage_driver,
            contract_id,
        }
    }

    fn db_key(&self, key: DBKey) -> Vec<u8> {
        db_key(key, &self.contract_id)
    }

    pub fn build_state(&mut self) {

        let (contracts, dependencies) = {
            let mut working_dir = match self.storage_driver {
                StorageDriver::Filesystem(ref config) => config.working_dir.clone(),
            };
            working_dir.push("stacks");
            let mut options = Options::default();
            options.create_if_missing(true);
            let contract_id = self.contract_id.clone();
            let db = DB::open_for_read_only(&options, working_dir, true).unwrap();
    
            // Get dependencies
            let mut interpreter =
                ClarityInterpreter::new(StandardPrincipalData::transient(), 2, vec![]);
    
            let mut contracts: BTreeMap<String, ContractInstanciation> = BTreeMap::new();
            let mut dependencies: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut queue = VecDeque::new();
    
            queue.push_front(contract_id.clone());
    
            while let Some(contract_id) = queue.pop_front() {
                let (_, deps) = match contracts.get(&contract_id) {
                    Some(entry) => (entry.clone(), Vec::new()),
                    None => {
                        let bytes = db
                            .get(&contract_id.as_bytes())
                            .expect("Unable to hit contract storage")
                            .expect("Unable to retrieve contract");
                        let contract_instance = serde_json::from_slice::<ContractInstanciation>(&bytes)
                            .expect("Unable to deserialize contract");
    
                        let deps = interpreter
                            .detect_dependencies(
                                contract_id.to_string(),
                                contract_instance.code.clone(),
                            )
                            .expect("Unable to retrieve contract dependencies");
    
                        contracts.insert(contract_id.to_string(), contract_instance.clone());
                        (contract_instance, deps)
                    }
                };
    
                if deps.len() > 0 {
                    dependencies.insert(
                        contract_id.to_string(),
                        deps.clone().into_iter().map(|c| c.to_string()).collect(),
                    );
                    for contract_id in deps.into_iter() {
                        queue.push_back(contract_id.to_string());
                    }
                    queue.push_back(contract_id.to_string());
                } else {
                    dependencies.insert(contract_id.to_string(), vec![]);
                }
            }
            (contracts, dependencies)
        };

        // Order the graph
        let ordered_contracts_ids = clarinet_lib::utils::order_contracts(&dependencies);

        // Build a SessionSettings struct from the contracts
        let mut settings = SessionSettings::default();
        settings.include_boot_contracts = vec!["costs-v2".to_string()];
        settings.costs_version = 2;
        settings.analysis = vec!["all".into()];

        let mut incremental_session = Session::new(settings);
        let mut full_analysis = BTreeMap::new();
        for contract_id in ordered_contracts_ids.into_iter() {
            let contract_instanciation = contracts.get(&contract_id).unwrap();
            let mut diagnostics = vec![];
            // Extract the AST, and try to move to the next contract if we throw an error:
            // we're trying to get as many errors as possible
            let contract_identifier = QualifiedContractIdentifier::parse(&contract_id).unwrap();
            let mut ast = match incremental_session
                .interpreter
                .build_ast(contract_identifier.clone(), 
                contract_instanciation.code.clone())
            {
                Ok(ast) => ast,
                Err((_, Some(diagnostic), _)) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
                _ => {
                    continue;
                }
            };

            // Run the analysis, and try to move to the next contract if we throw an error:
            // we're trying to get as many errors as possible
            let (annotations, mut annotation_diagnostics) = incremental_session
                .interpreter
                .collect_annotations(&ast, &contract_instanciation.code);
            diagnostics.append(&mut annotation_diagnostics);

            let (analysis, mut analysis_diagnostics) = match incremental_session.interpreter.run_analysis(
                contract_identifier.clone(),
                &mut ast,
                &annotations,
            ) {
                Ok(analysis) => analysis,
                Err((_, Some(diagnostic), _)) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
                _ => {
                    continue;
                }
            };
            diagnostics.append(&mut analysis_diagnostics);

            let _ = incremental_session.interpreter.save_contract(
                contract_identifier.clone(),
                &mut ast,
                contract_instanciation.code.clone(),
                analysis.clone(),
                false,
            );

            full_analysis.insert(
                contract_id.clone(),
                (diagnostics, analysis, ast),
            );
        }

        info!(self.log(), "ContractProcessor performed analysis {:?}", full_analysis);

        // Store artifacts
        {
            let db = contract_db_write(&self.storage_driver, &self.contract_id);
            let full_analysis_bytes = serde_json::to_vec(&full_analysis).expect("Unable to serialize block");
            db.put(&self.db_key(DBKey::FullAnalysis), full_analysis_bytes).unwrap();
            let (diags, analysis, ast) = full_analysis.get(&self.contract_id).unwrap();
            let interface = build_contract_interface(analysis);
            let interface_bytes = serde_json::to_vec(&interface).expect("Unable to serialize block");
            db.put(&self.db_key(DBKey::Interface), interface_bytes).unwrap();

            // todo(ludo): finer granularity
            // db.put(format!("{}::ast", contract_id).as_bytes(), full_analysis_bytes).unwrap();
            // db.put(format!("{}::diags", contract_id).as_bytes(), full_analysis_bytes).unwrap();
            // db.put(format!("{}::abi", contract_id).as_bytes(), full_analysis_bytes).unwrap();
        }
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
            ContractProcessorMessage::ProcessTransactionsBatch(transactions) => {
                info!(
                    self.ctx.log(),
                    "ContractProcessor processed transaction batch"
                );
                let mut changes = vec![];
                let mut custom_events = vec![];

                for tx in transactions.iter() {
                    for event in tx.metadata.receipt.events.iter() {
                        match event {
                            StacksTransactionEvent::DataVarSetEvent(event) => {
                                if event.contract_identifier == self.contract_id {
                                    changes.push(Changes::UpdateDataVar(&event.var, &event.hex_new_value, &tx.transaction_identifier.hash))
                                }
                            },
                            StacksTransactionEvent::DataMapInsertEvent(event) => {
                                if event.contract_identifier == self.contract_id {
                                    changes.push(Changes::InsertDataMapEntry(&event.map, (&event.hex_inserted_key, &event.hex_inserted_value), &tx.transaction_identifier.hash))
                                }
                            },
                            StacksTransactionEvent::DataMapUpdateEvent(event) => {
                                if event.contract_identifier == self.contract_id {
                                    changes.push(Changes::UpdateDataMapEntry(&event.map, (&event.hex_key, &event.hex_new_value), &tx.transaction_identifier.hash))
                                }
                            },
                            StacksTransactionEvent::DataMapDeleteEvent(event) => {
                                if event.contract_identifier == self.contract_id {
                                    changes.push(Changes::DeleteDataMapEntry(&event.map, &event.hex_deleted_key, &tx.transaction_identifier.hash))
                                }
                            },
                            StacksTransactionEvent::FTMintEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    let amount = u128::from_str_radix(&event.amount, 10)
                                        .expect("unable to parse amount");
                                    changes.push(Changes::ReceiveTokens(&event.asset_class_identifier, (&event.recipient, amount), &tx.transaction_identifier.hash))
                                }
                            }
                            StacksTransactionEvent::FTBurnEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    let amount = u128::from_str_radix(&event.amount, 10)
                                        .expect("unable to parse amount");
                                    changes.push(Changes::SendTokens(&event.asset_class_identifier, (&event.sender, amount), &tx.transaction_identifier.hash))
                                }
                            }
                            StacksTransactionEvent::FTTransferEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    let amount = u128::from_str_radix(&event.amount, 10)
                                        .expect("unable to parse amount");
                                    changes.push(Changes::SendTokens(&event.asset_class_identifier, (&event.sender, amount), &tx.transaction_identifier.hash));
                                    changes.push(Changes::ReceiveTokens(&event.asset_class_identifier, (&event.recipient, amount), &tx.transaction_identifier.hash))
                                }
                            }
                            StacksTransactionEvent::NFTMintEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    changes.push(Changes::ReceiveNFT(&event.asset_class_identifier, (&event.hex_asset_identifier, &event.recipient), &tx.transaction_identifier.hash))
                                }
                            }
                            StacksTransactionEvent::NFTBurnEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    changes.push(Changes::SendNFT(&event.asset_class_identifier, (&event.asset_identifier, &event.sender), &tx.transaction_identifier.hash))
                                }
                            }
                            StacksTransactionEvent::NFTTransferEvent(event) => {
                                if event.asset_class_identifier.starts_with(&self.contract_id) {
                                    changes.push(Changes::SendNFT(&event.asset_class_identifier, (&event.hex_asset_identifier, &event.sender), &tx.transaction_identifier.hash));
                                    changes.push(Changes::ReceiveNFT(&event.asset_class_identifier, (&event.hex_asset_identifier, &event.recipient), &tx.transaction_identifier.hash))
                                }                        
                            }
                            StacksTransactionEvent::SmartContractEvent(event) => {
                                if event.contract_identifier == self.contract_id {
                                    custom_events.push((event.clone(), tx.transaction_identifier.clone()));
                                }                   
                            }
                            StacksTransactionEvent::STXMintEvent(event) => {

                            }
                            StacksTransactionEvent::STXBurnEvent(event) => {
                                                            
                            }
                            StacksTransactionEvent::STXTransferEvent(event) => {
                                                            
                            }
                            StacksTransactionEvent::STXLockEvent(event) => {
                                
                            }
                        }
                    }
                }

                {
                    let db = contract_db_write(&self.storage_driver, &self.contract_id);
                    for change in changes.iter() {
                        match change {
                            Changes::UpdateDataVar(var, new_value, txid) => {
                                db.put(&self.db_key(DBKey::Var(var)), hex_bytes(new_value).unwrap()).expect("Unable to write");
                            },
                            Changes::InsertDataMapEntry(map, (new_key, new_value), txid) => {
                                db.put(&self.db_key(DBKey::MapEntry(map, new_key)), hex_bytes(new_value).unwrap()).expect("Unable to write");
                            },
                            Changes::DeleteDataMapEntry(map, deleted_key, txid) => {
                                db.delete(&self.db_key(DBKey::MapEntry(map, deleted_key))).expect("Unable to write");
                            },
                            Changes::UpdateDataMapEntry(map, (key, new_value), txid) => {
                                db.put(&self.db_key(DBKey::MapEntry(map, key)), hex_bytes(new_value).unwrap()).expect("Unable to write");
                            },
                            Changes::SendTokens(asset_id, (sender, value), txid) => {
                                let balance = match db.get(&self.db_key(DBKey::FT(asset_id, sender))) {
                                    Ok(Some(value)) => u128::from_str_radix(&String::from_utf8(value).unwrap(), 10).unwrap(),
                                    Ok(None) => 0,
                                    Err(e) => panic!("Operational problem encountered: {}", e),
                                };
                                db.put(&self.db_key(DBKey::FT(asset_id, sender)), (balance - value).to_string()).expect("Unable to write");
                            },
                            Changes::ReceiveTokens(asset_id, (recipient, value), txid) => {
                                let balance = match db.get(&self.db_key(DBKey::FT(asset_id, recipient))) {
                                    Ok(Some(value)) => u128::from_str_radix(&String::from_utf8(value).unwrap(), 10).unwrap(),
                                    Ok(None) => 0,
                                    Err(e) => panic!("Operational problem encountered: {}", e),
                                };
                                db.put(&self.db_key(DBKey::FT(asset_id, recipient)), (balance + value).to_string()).expect("Unable to write");
                            },
                            Changes::SendNFT(asset_class_id, (asset_id, sender), txid) => {
                                db.delete(&self.db_key(DBKey::NFT(asset_class_id, asset_id))).expect("Unable to write");
                            },
                            Changes::ReceiveNFT(asset_class_id, (asset_id, recipient), txid) => {
                                db.put(&self.db_key(DBKey::NFT(asset_class_id, asset_id)), recipient).expect("Unable to write");
                            },
                        }
                    }    
                }

                self.contract_processor_port.trigger(
                    ContractProcessorEvent::TransactionsBatchProcessed(self.contract_id.clone(), custom_events),
                )
            }
            ContractProcessorMessage::RollbackTransactionsBatch(transactions) => {}
            ContractProcessorMessage::Exit => {}
        };
        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
