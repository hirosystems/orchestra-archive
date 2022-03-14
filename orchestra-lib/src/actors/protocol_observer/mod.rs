mod update_api_generator;
mod update_state_explorer;

use clarinet_lib::clarity_repl::clarity::codec::StacksMessageCodec;
use clarinet_lib::clarity_repl::clarity::{Value};
use clarinet_lib::clarity_repl::clarity::analysis::{ContractAnalysis};
use clarinet_lib::clarity_repl::clarity::ast::ContractAST;
use clarinet_lib::clarity_repl::clarity::diagnostic::Diagnostic;
pub use update_state_explorer::UpdateStateExplorer;

use crate::datastore::blocks;
use crate::datastore::contracts::{contract_db_read, db_key, DBKey};
use crate::datastore::StorageDriver;
use crate::types::{
    Contract, FieldValues, FieldValuesRequest, FieldValuesResponse, FtValues, MapValues, NftValues,
    ProtocolObserverConfig, ProtocolRegistration, VarValues, ProtocolObserverId,
};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{
    ContractInterface, ContractInterfaceAtomType
};
use clarinet_lib::types::events::StacksTransactionEvent;
use clarinet_lib::types::{BitcoinBlockData, StacksBlockData};
use clarinet_lib::types::{BlockIdentifier, StacksTransactionData, TransactionIdentifier};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::build_contract_interface;
use clarinet_lib::clarity_repl::clarity::types::{
    QualifiedContractIdentifier, StandardPrincipalData,
};
use clarinet_lib::clarity_repl::clarity::util::hash::hex_bytes;
use clarinet_lib::clarity_repl::repl::settings::InitialContract;
use clarinet_lib::clarity_repl::repl::{ClarityInterpreter, Session, SessionSettings};
use clarinet_lib::types::events::{SmartContractEventData};
use super::block_store_manager::ContractInstanciation;

use kompact::prelude::*;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use rocksdb::{Options, DB};
use serde_json::map::Map;
use std::io::Cursor;
use std::sync::mpsc::Sender;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug)]
pub enum ProtocolObserverMessage {
    ProcessTransaction(StacksTransactionData),
    RollbackTransaction(StacksTransactionData),
    RequestFieldValues(FieldValuesRequest),
    GetInterfaces(Sender<ProtocolRegistration>),
    Exit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolObserverEvent {
    ContractsProcessed(ProtocolObserverId, BTreeMap<String, (ContractAnalysis, ContractAST, ContractInterface, BlockIdentifier)>),
}

pub struct ProtocolObserverPort;

impl Port for ProtocolObserverPort {
    type Indication = ProtocolObserverEvent;
    type Request = Never;
}

#[derive(ComponentDefinition)]
pub struct ProtocolObserver {
    ctx: ComponentContext<Self>,
    protocol_observer_port: ProvidedPort<ProtocolObserverPort>,
    config: ProtocolObserverConfig,
    storage_driver: StorageDriver,
}

ignore_requests!(ProtocolObserverPort, ProtocolObserver);

impl ProtocolObserver {
    pub fn new(storage_driver: StorageDriver, config: ProtocolObserverConfig) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            protocol_observer_port: ProvidedPort::uninitialised(),
            storage_driver,
            config,
        }
    }

    pub fn build_state(&mut self) {
        let (contracts, dependencies) = {

            let mut working_dir = match self.storage_driver {
                StorageDriver::Filesystem(ref config) => config.working_dir.clone(),
            };
            working_dir.push("stacks");
            let mut options = Options::default();
            options.create_if_missing(true);
            // Todo: re-approach this
            let db = DB::open_for_read_only(&options, working_dir, true).unwrap();

            // Get dependencies
            let settings = clarinet_lib::clarity_repl::repl::Settings::default();
            let mut interpreter =
                ClarityInterpreter::new(StandardPrincipalData::transient(), settings);

            let mut contracts: BTreeMap<String, ContractInstanciation> = BTreeMap::new();
            let mut dependencies: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut queue = VecDeque::new();
            for (contract_id, _) in self.config.contracts.iter() {
                queue.push_back(contract_id.to_string());
            } 

            while let Some(contract_id) = queue.pop_front() {
        
                if contracts.get(&contract_id).is_some() {
                    // Already handled, pursue dequeuing
                    continue;
                }

                let bytes = db
                    .get(&contract_id.as_bytes())
                    .expect("Unable to hit contract storage")
                    .expect(&format!("Unable to retrieve contract {}", contract_id));
                let contract_instance =
                    serde_json::from_slice::<ContractInstanciation>(&bytes)
                        .expect("Unable to deserialize contract");
                let deps = interpreter
                    .detect_dependencies(
                        contract_id.to_string(),
                        contract_instance.code.clone(),
                        2,
                    )
                    .expect("Unable to retrieve contract dependencies")
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>();

                contracts.insert(contract_id.to_string(), contract_instance.clone());
                
                dependencies.insert(
                    contract_id.to_string(),
                    deps.clone(),
                );

                if deps.len() > 0 {
                    info!(
                        self.log(),
                        "Dependencies 3: {:?} {:?}", contract_id, deps
                    );
    
                    for contract_id in deps.into_iter() {
                        queue.push_back(contract_id.to_string());
                    }
                    queue.push_back(contract_id.to_string());
                } else {
                    info!(
                        self.log(),
                        "Dependencies 4: {:?} {:?}", contract_id, deps
                    );
    
                    if !dependencies.contains_key(&contract_id) {
                        info!(
                            self.log(),
                            "Dependencies 5: {:?} {:?}", contract_id, deps
                        );
    
                        dependencies.insert(contract_id.to_string(), vec![]);
                    }
                }
            }
            (contracts, dependencies)
        };

        // Order the graph
        let ordered_contracts_ids = clarinet_lib::utils::order_contracts(&dependencies);

        // Build a SessionSettings struct from the contracts
        let mut settings = SessionSettings::default();
        settings.include_boot_contracts = vec!["costs-v2".to_string()];
        settings.repl_settings.costs_version = 2;

        let mut incremental_session = Session::new(settings);
        let mut full_analysis = BTreeMap::new();
        info!(
            self.log(),
            "Starting analysis of protocol {} with dependencies {:?}", self.config.project.name, ordered_contracts_ids
        );
        for contract_id in ordered_contracts_ids.into_iter() {
            let contract_instanciation = contracts.get(&contract_id).unwrap();
            let mut diagnostics = vec![];
            // Extract the AST, and try to move to the next contract if we throw an error:
            // we're trying to get as many errors as possible
            let contract_identifier = QualifiedContractIdentifier::parse(&contract_id).unwrap();
            let (mut ast, mut diags, success) = incremental_session.interpreter.build_ast(
                contract_identifier.clone(),
                contract_instanciation.code.clone(),
                2,
            );
            
            if !success {
                diagnostics.append(&mut diags);
                warn!(
                    self.log(),
                    "Errors {:?}", diagnostics
                );            
                continue;
            }

            // Run the analysis, and try to move to the next contract if we throw an error:
            // we're trying to get as many errors as possible
            let (annotations, mut annotation_diagnostics) = incremental_session
                .interpreter
                .collect_annotations(&ast, &contract_instanciation.code);
            diagnostics.append(&mut annotation_diagnostics);

            let (analysis, mut analysis_diagnostics) = match incremental_session
                .interpreter
                .run_analysis(contract_identifier.clone(), &mut ast, &annotations)
            {
                Ok(analysis) => analysis,
                Err((_, Some(diagnostic), _)) => {
                    diagnostics.push(diagnostic);
                    warn!(
                        self.log(),
                        "Errors {:?}", diagnostics
                    );            
                    continue;
                }
                _ => {
                    warn!(
                        self.log(),
                        "Silent Error 1"
                    );
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

            let interface = build_contract_interface(&analysis);
            let contract_instanciation = contracts.get(&contract_id).unwrap();

            full_analysis.insert(contract_id.clone(), (analysis, ast, interface, contract_instanciation.block_identifier.clone()));
        }

        info!(
            self.log(),
            "ProtocolObserver performed analysis for {}", self.config.project.name
        );


        self.protocol_observer_port.trigger(
            ProtocolObserverEvent::ContractsProcessed(self.config.identifier.clone(), full_analysis)
        );
    }
}

impl ComponentLifecycle for ProtocolObserver {
    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ProtocolObserver starting");
        self.build_state();
        Handled::Ok
    }
}

impl Actor for ProtocolObserver {
    type Message = ProtocolObserverMessage;

    fn receive_local(&mut self, msg: ProtocolObserverMessage) -> Handled {
        info!(self.ctx.log(), "ProtocolObserver received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ProtocolObserver")
            .install_simple()
            .unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ProtocolObserverMessage::ProcessTransaction(tx) => {}
            ProtocolObserverMessage::RollbackTransaction(tx) => {}
            ProtocolObserverMessage::GetInterfaces(tx) => {
                let mut contracts = vec![];
                for (contract_id, _) in self.config.contracts.iter() {
                    let contract_id = contract_id.to_string();

                    let db = contract_db_read(&self.storage_driver, &contract_id);

                    let bytes = db
                        .get(db_key(DBKey::Interface, &contract_id))
                        .unwrap()
                        .unwrap();
                    let interface = serde_json::from_slice::<ContractInterface>(&bytes)
                        .expect("Unable to deserialize contract");

                    contracts.push(Contract {
                        contract_identifier: contract_id,
                        interface,
                    })
                }
                tx.send(ProtocolRegistration { contracts }).unwrap()
            }
            ProtocolObserverMessage::RequestFieldValues(request) => {
                let db = contract_db_read(&self.storage_driver, &request.contract_identifier);

                let bytes = db
                    .get(db_key(DBKey::Interface, &request.contract_identifier))
                    .unwrap()
                    .unwrap();
                let interface = serde_json::from_slice::<ContractInterface>(&bytes)
                    .expect("Unable to deserialize contract");

                let mut field = None;
                for var in interface.variables.iter() {
                    if var.name == request.field_name {
                        let value = match db
                            .get(db_key(DBKey::Var(&var.name), &request.contract_identifier))
                        {
                            Ok(None) => Value::none(),
                            Ok(Some(bytes)) => {
                                match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
                                    Ok(value) => value,
                                    Err(_) => Value::none(),
                                }
                            }
                            _ => Value::none(),
                        };

                        let events_key =
                            db_key(DBKey::VarEventScan(&var.name), &request.contract_identifier);
                        let key = String::from_utf8(events_key.to_vec()).unwrap();
                        warn!(self.ctx().log(), "Events key: {:?}", key);

                        let iter = db.prefix_iterator(&events_key);
                        let mut events = vec![];
                        for (key, value) in iter {
                            if key.starts_with(&events_key) {
                                let remainder =
                                    String::from_utf8(key[events_key.len()..].to_vec()).unwrap();
                                let order = remainder.split("/").collect::<Vec<&str>>();
                                let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                let event =
                                    serde_json::from_slice::<StacksTransactionEvent>(&value)
                                        .expect("Unable to deserialize contract");
                                events.push((event, block_index, event_index));
                            }
                        }
                        events.sort_by(|(_, b1, b2), (_, a1, a2)| {
                            (a1 * 100 + a2).cmp(&(b1 * 100 + b2))
                        });
                        warn!(self.ctx().log(), "Events: {:?}", events);

                        field = Some(FieldValues::Var(VarValues {
                            value: format!("{}", value),
                            value_type: var.type_f.clone(),
                            events,
                            events_page_index: 0,
                            events_page_size: 0,
                        }));
                    }
                }

                if field.is_none() {
                    for map in interface.maps.iter() {
                        if map.name == request.field_name {
                            let value_key =
                                db_key(DBKey::MapScan(&map.name), &request.contract_identifier);
                            let iter = db.prefix_iterator(&value_key);
                            let mut entries = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&value_key) {
                                    let decoded_key = match Value::consensus_deserialize(
                                        &mut Cursor::new(&key[value_key.len()..]),
                                    ) {
                                        Ok(value) => value,
                                        Err(_) => Value::none(),
                                    };
                                    let formatted_key = match decoded_key {
                                        Value::Tuple(pairs) => {
                                            let mut map = Map::new();
                                            for (key, value) in pairs.data_map.into_iter() {
                                                map.insert(
                                                    key.to_string(),
                                                    format!("{}", value).into(),
                                                );
                                            }
                                            json!(map).to_string()
                                        }
                                        _ => format!("{}", decoded_key),
                                    };

                                    let decoded_value = match Value::consensus_deserialize(
                                        &mut Cursor::new(&value),
                                    ) {
                                        Ok(decoded_value) => decoded_value,
                                        Err(e) => {
                                            error!(
                                                self.ctx.log(),
                                                "Error decoding clarity value {}", e
                                            );
                                            Value::none()
                                        }
                                    };
                                    let formatted_value = match decoded_value {
                                        Value::Tuple(pairs) => {
                                            let mut map = Map::new();
                                            for (key, value) in pairs.data_map.into_iter() {
                                                map.insert(
                                                    key.to_string(),
                                                    format!("{}", value).into(),
                                                );
                                            }
                                            json!(map).to_string()
                                        }
                                        _ => format!("{}", decoded_value),
                                    };
                                    entries.push((
                                        (formatted_key, formatted_value),
                                        BlockIdentifier {
                                            hash: "0".into(),
                                            index: 0,
                                        },
                                        TransactionIdentifier { hash: "0".into() },
                                    ))
                                }
                            }

                            let events_key = db_key(
                                DBKey::MapEventScan(&map.name),
                                &request.contract_identifier,
                            );
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);

                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder =
                                        String::from_utf8(key[events_key.len()..].to_vec())
                                            .unwrap();
                                    let order = remainder.split("/").collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event =
                                        serde_json::from_slice::<StacksTransactionEvent>(&value)
                                            .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| {
                                (a1 * 100 + a2).cmp(&(b1 * 100 + b2))
                            });
                            warn!(self.ctx().log(), "Events: {:?}", events);

                            field = Some(FieldValues::Map(MapValues {
                                entries,
                                entries_page_size: 0,
                                entries_page_index: 0,
                                key_type: map.key.clone(),
                                value_type: map.value.clone(),
                                events,
                                events_page_index: 0,
                                events_page_size: 0,
                            }));
                        }
                    }
                }
                if field.is_none() {
                    for nft in interface.non_fungible_tokens.iter() {
                        if nft.name == request.field_name {
                            let asset_id = format!("{}::{}", request.contract_identifier, nft.name);
                            let values_key =
                                db_key(DBKey::NFTScan(&asset_id), &request.contract_identifier);

                            let iter = db.prefix_iterator(&values_key);
                            let mut tokens = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&values_key) {
                                    let decoded_key = match Value::consensus_deserialize(
                                        &mut Cursor::new(&key[values_key.len()..]),
                                    ) {
                                        Ok(value) => value,
                                        Err(_) => Value::none(),
                                    };
                                    let owner = String::from_utf8(value.to_vec()).unwrap();

                                    let asset = match decoded_key {
                                        Value::Tuple(pairs) => {
                                            let mut map = Map::new();
                                            for (key, value) in pairs.data_map.into_iter() {
                                                map.insert(
                                                    key.to_string(),
                                                    format!("{}", value).into(),
                                                );
                                            }
                                            json!(map).to_string()
                                        }
                                        _ => format!("{}", decoded_key),
                                    };

                                    tokens.push((
                                        (asset, owner),
                                        BlockIdentifier {
                                            hash: "0".into(),
                                            index: 0,
                                        },
                                        TransactionIdentifier { hash: "0".into() },
                                    ))
                                }
                            }

                            let events_key = db_key(
                                DBKey::NFTEventScan(&asset_id),
                                &request.contract_identifier,
                            );
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);

                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder =
                                        String::from_utf8(key[events_key.len()..].to_vec())
                                            .unwrap();
                                    let order = remainder.split("/").collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event =
                                        serde_json::from_slice::<StacksTransactionEvent>(&value)
                                            .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| {
                                (a1 * 100 + a2).cmp(&(b1 * 100 + b2))
                            });
                            warn!(self.ctx().log(), "Events: {:?}", events);

                            field = Some(FieldValues::Nft(NftValues {
                                tokens,
                                tokens_page_size: 0,
                                tokens_page_index: 0,
                                token_type: nft.type_f.clone(),
                                events,
                                events_page_index: 0,
                                events_page_size: 0,
                            }));
                        }
                    }
                }
                if field.is_none() {
                    for ft in interface.fungible_tokens.iter() {
                        if ft.name == request.field_name {
                            let asset_id = format!("{}::{}", request.contract_identifier, ft.name);
                            let values_key =
                                db_key(DBKey::FTScan(&asset_id), &request.contract_identifier);

                            let iter = db.prefix_iterator(&values_key);
                            let mut balances = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&values_key) {
                                    let owner = String::from_utf8(key[values_key.len()..].to_vec())
                                        .unwrap();
                                    let balance = String::from_utf8(value.to_vec()).unwrap();

                                    balances.push((
                                        (owner, balance),
                                        BlockIdentifier {
                                            hash: "0".into(),
                                            index: 0,
                                        },
                                        TransactionIdentifier { hash: "0".into() },
                                    ))
                                }
                            }
                            let events_key =
                                db_key(DBKey::FTEventScan(&asset_id), &request.contract_identifier);
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);

                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder =
                                        String::from_utf8(key[events_key.len()..].to_vec())
                                            .unwrap();
                                    let order = remainder.split("/").collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event =
                                        serde_json::from_slice::<StacksTransactionEvent>(&value)
                                            .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| {
                                (a1 * 100 + a2).cmp(&(b1 * 100 + b2))
                            });
                            warn!(self.ctx().log(), "Events: {:?}", events);

                            field = Some(FieldValues::Ft(FtValues {
                                balances,
                                balances_page_size: 0,
                                balances_page_index: 0,
                                events,
                                events_page_index: 0,
                                events_page_size: 0,
                            }));
                        }
                    }
                }

                // Get eventual latest blocks (bitcoin + stacks)
                let stacks_db = blocks::stacks_blocks_db_read(&self.storage_driver);
                let stacks_tip = u64::from_be_bytes(
                    stacks_db
                        .get("tip".as_bytes())
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                );
                let mut stacks_blocks = vec![];
                warn!(
                    self.ctx().log(),
                    "Will be looking for stacks blocks in range {:?}",
                    request.stacks_block_identifier.index..stacks_tip
                );
                for missing_block in request.stacks_block_identifier.index..stacks_tip {
                    let hash = stacks_db
                        .get(&missing_block.to_be_bytes())
                        .unwrap()
                        .unwrap();
                    let block_bytes = stacks_db
                        .get(&format!("hash:{}", String::from_utf8(hash).unwrap()))
                        .unwrap()
                        .unwrap();
                    let block = serde_json::from_slice::<StacksBlockData>(&block_bytes)
                        .expect("Unable to deserialize contract");
                    stacks_blocks.push(block);
                }
                warn!(self.ctx().log(), "Found {:?}", stacks_blocks);

                let bitcoin_db = blocks::bitcoin_blocks_db_read(&self.storage_driver);
                let bitcoin_tip = u64::from_be_bytes(
                    bitcoin_db
                        .get("tip".as_bytes())
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                );
                let mut bitcoin_blocks = vec![];
                // for missing_block in request..index..bitcoin_tip {
                //     let hash = stacks_db.get(&missing_block.to_be_bytes()).unwrap().unwrap();
                //     let block_bytes = stacks_db.get(&format!("hash:{}", String::from_utf8(hash).unwrap())).unwrap().unwrap();
                //     let block = serde_json::from_slice::<BitcoinBlockData>(&block_bytes)
                //                         .expect("Unable to deserialize contract");
                //     bitcoin_blocks.push(block);
                // }

                let response = FieldValuesResponse {
                    bitcoin_blocks,
                    stacks_blocks,
                    contract_identifier: request.contract_identifier.clone(),
                    field_name: request.field_name.clone(),
                    values: field.unwrap(),
                };
                request.tx.send(response).unwrap();
            }
            ProtocolObserverMessage::Exit => {}
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}
