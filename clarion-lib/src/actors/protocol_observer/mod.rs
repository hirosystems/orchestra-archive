mod update_api_generator;
mod update_state_explorer;

use clarinet_lib::clarity_repl::clarity::Value;
use clarinet_lib::clarity_repl::clarity::codec::StacksMessageCodec;
pub use update_state_explorer::UpdateStateExplorer;

use clarinet_lib::types::{StacksTransactionData, BlockIdentifier, TransactionIdentifier};
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{ContractInterface, ContractInterfaceAtomType};
use clarinet_lib::types::events::{StacksTransactionEvent};
use crate::datastore::contracts::{db_key, DBKey, contract_db_read};
use crate::datastore::StorageDriver;
use crate::types::{Contract, ProtocolObserverConfig, FieldValuesRequest, FieldValuesResponse, FieldValues, VarValues, MapValues, NftValues, FtValues, ProtocolRegistration};
use serde_json::map::Map;
use std::io::Cursor;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum ProtocolObserverMessage {
    ProcessTransaction(StacksTransactionData),
    RollbackTransaction(StacksTransactionData),
    RequestFieldValues(FieldValuesRequest),
    GetInterfaces(Sender<ProtocolRegistration>),
    Exit,
}

#[derive(ComponentDefinition)]
pub struct ProtocolObserver {
    ctx: ComponentContext<Self>,
    config: ProtocolObserverConfig,
    storage_driver: StorageDriver,
}

impl ProtocolObserver {
    pub fn new(storage_driver: StorageDriver, config: ProtocolObserverConfig) -> Self {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
        Self {
            ctx: ComponentContext::uninitialised(),
            storage_driver,
            config,
        }
    }
}

impl ComponentLifecycle for ProtocolObserver {

    fn on_start(&mut self) -> Handled {
        info!(self.log(), "ProtocolObserver starting");

        Handled::Ok
    }
}

impl Actor for ProtocolObserver {
    type Message = ProtocolObserverMessage;

    fn receive_local(&mut self, msg: ProtocolObserverMessage) -> Handled {
        info!(self.ctx.log(), "ProtocolObserver received message");

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_service_name("ProtocolObserver")
            .install_simple().unwrap();
        let mut span = tracer.start("handle message");

        match msg {
            ProtocolObserverMessage::ProcessTransaction(tx) => {
                
            },
            ProtocolObserverMessage::RollbackTransaction(tx) => {

            },
            ProtocolObserverMessage::GetInterfaces(tx) => {
                let mut contracts = vec![];
                for (contract_id, _) in self.config.contracts.iter() {
                    let contract_id = contract_id.to_string();

                    let db = contract_db_read(&self.storage_driver, &contract_id);

                    let bytes = db.get(db_key(DBKey::Interface, &contract_id)).unwrap().unwrap();
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

                let bytes = db.get(db_key(DBKey::Interface, &request.contract_identifier)).unwrap().unwrap();
                let interface = serde_json::from_slice::<ContractInterface>(&bytes)
                    .expect("Unable to deserialize contract");

                let mut field = None;
                for var in interface.variables.iter() {
                    if var.name == request.field_name {
                        let value = match db.get(db_key(DBKey::Var(&var.name), &request.contract_identifier)) {
                            Ok(None) => Value::none(),
                            Ok(Some(bytes)) => match Value::consensus_deserialize(&mut Cursor::new(&bytes)) {
                                Ok(value) => value,
                                Err(_) => Value::none(),
                            }
                            _ => Value::none(),
                        };

                        let events_key = db_key(DBKey::VarEventScan(&var.name), &request.contract_identifier);
                        let key = String::from_utf8(events_key.to_vec()).unwrap();
                        warn!(self.ctx().log(), "Events key: {:?}", key);

                        let iter = db.prefix_iterator(&events_key);
                        let mut events = vec![];
                        for (key, value) in iter {
                            if key.starts_with(&events_key) {
                                let remainder = String::from_utf8(key[events_key.len()..].to_vec()).unwrap();
                                let order = remainder.split("/")
                                    .collect::<Vec<&str>>();
                                let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                let event = serde_json::from_slice::<StacksTransactionEvent>(&value)
                                    .expect("Unable to deserialize contract");
                                events.push((event, block_index, event_index));    
                            }
                        }
                        events.sort_by(|(_, b1, b2), (_, a1, a2)| (a1*100+a2).cmp(&(b1*100+b2)));
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
                            let value_key = db_key(DBKey::MapScan(&map.name), &request.contract_identifier);
                            let iter = db.prefix_iterator(&value_key);
                            let mut entries = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&value_key) {
                                    let decoded_key = match Value::consensus_deserialize(&mut Cursor::new(&key[value_key.len()..])) {
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
                                        _ => format!("{}", decoded_key)
                                    };

                                    let decoded_value = match Value::consensus_deserialize(&mut Cursor::new(&value)) {
                                        Ok(decoded_value) => decoded_value,
                                        Err(e) => {
                                            error!(self.ctx.log(), "Error decoding clarity value {}", e);
                                            Value::none()
                                        }
                                    };
                                    let formatted_value = match decoded_value {
                                        Value::Tuple(pairs) => {
                                            let mut map = Map::new();
                                            for (key, value) in pairs.data_map.into_iter() {
                                                map.insert(key.to_string(), format!("{}", value).into());
                                            }
                                            json!(map).to_string()
                                        }
                                        _ => format!("{}", decoded_value)
                                    };
                                    entries.push(((formatted_key, formatted_value), BlockIdentifier {
                                        hash: "0".into(),
                                        index: 0
                                    }, TransactionIdentifier {
                                        hash: "0".into()
                                    }))
                                }
                            }

                            let events_key = db_key(DBKey::MapEventScan(&map.name), &request.contract_identifier);
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);
    
                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder = String::from_utf8(key[events_key.len()..].to_vec()).unwrap();
                                    let order = remainder.split("/")
                                        .collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event = serde_json::from_slice::<StacksTransactionEvent>(&value)
                                        .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));    
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| (a1*100+a2).cmp(&(b1*100+b2)));
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
                            let values_key = db_key(DBKey::NFTScan(&asset_id), &request.contract_identifier);

                            let iter = db.prefix_iterator(&values_key);
                            let mut tokens = vec![];
                            for (key, value) in iter {

                                if key.starts_with(&values_key) {
                                    let decoded_key = match Value::consensus_deserialize(&mut Cursor::new(&key[values_key.len()..])) {
                                        Ok(value) => value,
                                        Err(_) => Value::none(),
                                    };
                                    let owner = String::from_utf8(value.to_vec()).unwrap();

                                    let asset = match decoded_key {
                                        Value::Tuple(pairs) => {
                                            let mut map = Map::new();
                                            for (key, value) in pairs.data_map.into_iter() {
                                                map.insert(key.to_string(), format!("{}", value).into());
                                            }
                                            json!(map).to_string()
                                        }
                                        _ => format!("{}", decoded_key)
                                    };

                                    tokens.push(((asset, owner), BlockIdentifier {
                                        hash: "0".into(),
                                        index: 0
                                    }, TransactionIdentifier {
                                        hash: "0".into()
                                    }))
                                }
                            }

                            let events_key = db_key(DBKey::NFTEventScan(&asset_id), &request.contract_identifier);
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);
    
                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder = String::from_utf8(key[events_key.len()..].to_vec()).unwrap();
                                    let order = remainder.split("/")
                                        .collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event = serde_json::from_slice::<StacksTransactionEvent>(&value)
                                        .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));    
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| (a1*100+a2).cmp(&(b1*100+b2)));
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
                            let values_key = db_key(DBKey::FTScan(&asset_id), &request.contract_identifier);

                            let iter = db.prefix_iterator(&values_key);
                            let mut balances = vec![];
                            for (key, value) in iter {

                                if key.starts_with(&values_key) {
                                    let owner = String::from_utf8(key[values_key.len()..].to_vec()).unwrap();
                                    let balance = String::from_utf8(value.to_vec()).unwrap();

                                    balances.push(((owner, balance), BlockIdentifier {
                                        hash: "0".into(),
                                        index: 0
                                    }, TransactionIdentifier {
                                        hash: "0".into()
                                    }))
                                }
                            }
                            let events_key = db_key(DBKey::FTEventScan(&asset_id), &request.contract_identifier);
                            let key = String::from_utf8(events_key.clone()).unwrap();
                            warn!(self.ctx().log(), "Events key: {:?}", key);
    
                            let iter = db.prefix_iterator(&events_key);
                            let mut events = vec![];
                            for (key, value) in iter {
                                if key.starts_with(&events_key) {
                                    let remainder = String::from_utf8(key[events_key.len()..].to_vec()).unwrap();
                                    let order = remainder.split("/")
                                        .collect::<Vec<&str>>();
                                    let block_index = u64::from_str_radix(order[0], 10).unwrap();
                                    let event_index = u64::from_str_radix(order[1], 10).unwrap();

                                    let event = serde_json::from_slice::<StacksTransactionEvent>(&value)
                                        .expect("Unable to deserialize contract");
                                    events.push((event, block_index, event_index));    
                                }
                            }
                            events.sort_by(|(_, b1, b2), (_, a1, a2)| (a1*100+a2).cmp(&(b1*100+b2)));
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

                if field.is_none() {
                    field = Some(FieldValues::Var(VarValues {
                        value: "201".to_string(),
                        value_type: ContractInterfaceAtomType::uint128,
                        events: vec![],
                        events_page_size: 0,
                        events_page_index: 0,
                    }),);
                }

                let response = FieldValuesResponse {
                    contract_identifier: request.contract_identifier.clone(),
                    field_name: request.field_name.clone(),
                    values: field.unwrap(),
                };
                request.tx.send(response).unwrap();
            }
            ProtocolObserverMessage::Exit => {

            }
        };

        span.end();
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        unimplemented!()
    }
}

