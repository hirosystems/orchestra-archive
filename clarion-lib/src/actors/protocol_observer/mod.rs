mod update_api_generator;
mod update_state_explorer;

pub use update_state_explorer::UpdateStateExplorer;

use clarinet_lib::types::StacksTransactionData;
use kompact::prelude::*;
use opentelemetry::trace::{Tracer, Span};
use opentelemetry::global;
use rocksdb::{DB, Options};
use clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{ContractInterface, ContractInterfaceAtomType};
use crate::datastore::contracts::{db_key, DBKey, contract_db_read};
use crate::datastore::StorageDriver;
use crate::types::{Contract, ProtocolObserverConfig, FieldValuesRequest, FieldValuesResponse, FieldValues, VarValues, MapValues, NftValues, FtValues, ProtocolRegistration};

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
                        field = Some(FieldValues::Var(VarValues {
                            value: "".into(),
                            value_type: var.type_f.clone(),
                            events: vec![],
                            events_page_index: 0,
                            events_page_size: 0,
                        }));
                    }
                }
                if field.is_none() {
                    for map in interface.maps.iter() {
                        if map.name == request.field_name {
                            field = Some(FieldValues::Map(MapValues {
                                pairs: vec![],
                                pairs_page_size: 0,
                                pairs_page_index: 0,
                                key_type: map.key.clone(),
                                value_type: map.value.clone(),
                                events: vec![],
                                events_page_index: 0,
                                events_page_size: 0,
                            }));
                        }
                    }    
                }
                if field.is_none() {
                    for nft in interface.non_fungible_tokens.iter() {
                        if nft.name == request.field_name {
                            field = Some(FieldValues::Nft(NftValues {
                                tokens: vec![],
                                tokens_page_size: 0,
                                tokens_page_index: 0,
                                token_type: nft.type_f.clone(),
                                events: vec![],
                                events_page_index: 0,
                                events_page_size: 0,
                            }));
                        }
                    }    
                }
                if field.is_none() {
                    for ft in interface.fungible_tokens.iter() {
                        if ft.name == request.field_name {
                            field = Some(FieldValues::Ft(FtValues {
                                balances: vec![],
                                balances_page_size: 0,
                                balances_page_index: 0,
                                events: vec![],
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