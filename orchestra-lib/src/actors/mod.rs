mod block_store_manager;
mod contract_processor;
mod protocol_observer;
mod supervisor;

pub use block_store_manager::{BlockStoreManager, BlockStoreManagerMessage};
pub use contract_processor::{ContractProcessor, ContractProcessorMessage};
pub use protocol_observer::{ProtocolObserver, ProtocolObserverMessage};
pub use supervisor::{OrchestraSupervisor, OrchestraSupervisorMessage};

use kompact::prelude::*;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::datastore::StorageDriver;

pub fn run_supervisor(
    storage_driver: StorageDriver,
    supervisor_cmd_rx: Receiver<OrchestraSupervisorMessage>,
) -> Result<(), String> {
    match block_on(do_run_supervisor(storage_driver, supervisor_cmd_rx)) {
        Err(_e) => std::process::exit(1),
        Ok(res) => Ok(res),
    }
}

pub fn block_on<F, R>(future: F) -> R
where
    F: std::future::Future<Output = R>,
{
    let rt = clarinet_lib::utils::create_basic_runtime();
    rt.block_on(future)
}

pub async fn do_run_supervisor(
    storage_driver: StorageDriver,
    supervisor_cmd_rx: Receiver<OrchestraSupervisorMessage>,
) -> Result<(), String> {
    // let drain = slog::Discard;
    // let log  = slog::Logger::root(drain, o!());
    // let log = Logger::root(slog_term::term_compact().fuse(),
    // o!("version" => env!("CARGO_PKG_VERSION")));

    // info!(log, "Spawning supervisor");
    let system = KompactConfig::default().build().expect("system");
    let supervisor: Arc<Component<OrchestraSupervisor>> =
        system.create(|| OrchestraSupervisor::new(storage_driver));
    system.start(&supervisor);
    let supervisor_ref = supervisor.actor_ref();

    std::thread::spawn(move || {
        while let Ok(msg) = supervisor_cmd_rx.recv() {
            supervisor_ref.tell(msg);
        }
    });
    system.await_termination();
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::actors::OrchestraSupervisorMessage;
    use crate::datastore::StorageDriver;
    use clarinet_lib::types::{
        BlockIdentifier, StacksBlockData, StacksBlockMetadata, StacksContractDeploymentData,
        StacksTransactionData, StacksTransactionKind, StacksTransactionMetadata,
        StacksTransactionReceipt, TransactionIdentifier,
    };
    use opentelemetry::trace::{Span, SpanContext, StatusCode};
    use opentelemetry::KeyValue;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[derive(Debug)]
    struct MockedSpan {
        context: SpanContext,
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
        ) {
        }
        fn span_context(&self) -> &SpanContext {
            return &self.context;
        }
        fn is_recording(&self) -> bool {
            true
        }
        fn set_attribute(&mut self, _attribute: KeyValue) {}
        fn set_status(&mut self, _code: StatusCode, _message: String) {}
        fn update_name(&mut self, _new_name: String) {}
        fn end(&mut self) {}
        fn end_with_timestamp(&mut self, _timestamp: SystemTime) {}
    }

    fn transaction_contract_call_impacting_contract_id(
        contract_id: String,
        success: bool,
    ) -> StacksTransactionData {
        let mut mutated_contracts_radius = HashSet::new();
        mutated_contracts_radius.insert(contract_id);
        StacksTransactionData {
            transaction_identifier: TransactionIdentifier { hash: "0".into() },
            operations: vec![],
            metadata: StacksTransactionMetadata {
                success,
                result: "".into(),
                raw_tx: "0x00".to_string(),
                execution_cost: None,
                sender: "".into(),
                fee: 0,
                sponsor: None,
                kind: StacksTransactionKind::ContractCall,
                receipt: StacksTransactionReceipt {
                    mutated_contracts_radius,
                    mutated_assets_radius: HashSet::new(),
                    events: vec![],
                },
                description: "".into(),
            },
        }
    }

    fn transaction_contract_deployment(contract_id: String, code: &str) -> StacksTransactionData {
        let mut mutated_contracts_radius = HashSet::new();
        mutated_contracts_radius.insert(contract_id.clone());
        StacksTransactionData {
            transaction_identifier: TransactionIdentifier { hash: "0".into() },
            operations: vec![],
            metadata: StacksTransactionMetadata {
                success: true,
                result: "".into(),
                raw_tx: "0x00".to_string(),
                execution_cost: None,
                sender: "".into(),
                fee: 0,
                sponsor: None,
                kind: StacksTransactionKind::ContractDeployment(StacksContractDeploymentData {
                    contract_identifier: contract_id,
                    code: code.to_string(),
                }),
                receipt: StacksTransactionReceipt {
                    mutated_contracts_radius,
                    mutated_assets_radius: HashSet::new(),
                    events: vec![],
                },
                description: "".into(),
            },
        }
    }

    fn block_with_transactions(transactions: Vec<StacksTransactionData>) -> StacksBlockData {
        StacksBlockData {
            block_identifier: BlockIdentifier {
                index: 1,
                hash: "1".into(),
            },
            parent_block_identifier: BlockIdentifier {
                index: 0,
                hash: "0".into(),
            },
            timestamp: 0,
            transactions,
            metadata: StacksBlockMetadata {
                bitcoin_anchor_block_identifier: BlockIdentifier {
                    index: 0,
                    hash: "0".into(),
                },
                pox_cycle_index: 0,
                pox_cycle_position: 0,
                pox_cycle_length: 0,
            },
        }
    }

    #[test]
    fn spawn_integrated_supervisor() {
        use crate::actors::run_supervisor;
        use crate::types::{
            ContractSettings, ProjectMetadata, ProtocolObserverConfig, ProtocolObserverId,
        };
        use clarinet_lib::clarity_repl::clarity::types::{
            QualifiedContractIdentifier, StandardPrincipalData,
        };
        use clarinet_lib::types::events::*;
        use clarinet_lib::types::StacksChainEvent;
        use std::collections::BTreeMap;
        use std::convert::TryInto;
        use std::sync::mpsc::channel;
        use std::{thread, time};

        let mut contracts = BTreeMap::new();
        let test_contract_id = QualifiedContractIdentifier::new(
            StandardPrincipalData::transient(),
            "test".try_into().unwrap(),
        );
        let test_contract_settings = ContractSettings {
            state_explorer_enabled: true,
            api_generator_enabled: vec![],
        };
        contracts.insert(test_contract_id.clone(), test_contract_settings);

        let (tx, rx) = channel();
        let storage_driver = StorageDriver::tmpfs();
        let storage_driver_moved = storage_driver.clone();
        let handle = std::thread::spawn(|| run_supervisor(storage_driver_moved, rx));

        let orchestra_manifest = ProtocolObserverConfig {
            identifier: ProtocolObserverId(0),
            project: ProjectMetadata {
                name: "test".into(),
                authors: vec![],
                homepage: "".into(),
                license: "".into(),
                description: "".into(),
            },
            lambdas: vec![],
            contracts,
            manifest_path: PathBuf::new(),
        };

        let block = block_with_transactions(vec![transaction_contract_deployment(
            test_contract_id.to_string(),
            "(print \"hello world\")",
        )]);
        tx.send(OrchestraSupervisorMessage::ProcessStacksChainEvent(
            StacksChainEvent::ChainUpdatedWithBlock(block),
        ))
        .unwrap();

        let delay = time::Duration::from_millis(100);
        thread::sleep(delay);

        tx.send(OrchestraSupervisorMessage::RegisterProtocolObserver(
            orchestra_manifest,
        ))
        .unwrap();

        let mut transaction =
            transaction_contract_call_impacting_contract_id(test_contract_id.to_string(), true);
        transaction.metadata.receipt.events.append(&mut vec![
            StacksTransactionEvent::DataVarSetEvent(DataVarSetEventData {
                var: "my-var".into(),
                hex_new_value: "1".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                map: "my-map".into(),
                hex_inserted_key: "k1".into(),
                hex_inserted_value: "v1".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                map: "my-map".into(),
                hex_inserted_key: "k2".into(),
                hex_inserted_value: "v2".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                map: "my-map".into(),
                hex_inserted_key: "k3".into(),
                hex_inserted_value: "v3".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::DataMapUpdateEvent(DataMapUpdateEventData {
                map: "my-map".into(),
                hex_key: "k2".into(),
                hex_new_value: "v4".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::DataMapDeleteEvent(DataMapDeleteEventData {
                map: "my-map".into(),
                hex_deleted_key: "k3".into(),
                contract_identifier: test_contract_id.to_string(),
            }),
            StacksTransactionEvent::FTMintEvent(FTMintEventData {
                asset_class_identifier: format!("{}::my-ft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P01".into(),
                amount: "100".into(),
            }),
            StacksTransactionEvent::FTMintEvent(FTMintEventData {
                asset_class_identifier: format!("{}::my-ft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P02".into(),
                amount: "1".into(),
            }),
            StacksTransactionEvent::FTMintEvent(FTMintEventData {
                asset_class_identifier: format!("{}::my-ft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P03".into(),
                amount: "10000000".into(),
            }),
            StacksTransactionEvent::FTBurnEvent(FTBurnEventData {
                asset_class_identifier: format!("{}::my-ft", test_contract_id),
                sender: "S1G2081040G2081040G2081040G208105NK8P02".into(),
                amount: "1".into(),
            }),
            StacksTransactionEvent::FTTransferEvent(FTTransferEventData {
                asset_class_identifier: format!("{}::my-ft", test_contract_id),
                sender: "S1G2081040G2081040G2081040G208105NK8P01".into(),
                recipient: "S1G2081040G2081040G2081040G208105NK8P03".into(),
                amount: "100".into(),
            }),
            StacksTransactionEvent::NFTMintEvent(NFTMintEventData {
                asset_class_identifier: format!("{}::my-nft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P01".into(),
                asset_identifier: "".into(),
                hex_asset_identifier: "A".into(),
            }),
            StacksTransactionEvent::NFTMintEvent(NFTMintEventData {
                asset_class_identifier: format!("{}::my-nft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P02".into(),
                asset_identifier: "".into(),
                hex_asset_identifier: "B".into(),
            }),
            StacksTransactionEvent::NFTMintEvent(NFTMintEventData {
                asset_class_identifier: format!("{}::my-nft", test_contract_id),
                recipient: "S1G2081040G2081040G2081040G208105NK8P03".into(),
                asset_identifier: "".into(),
                hex_asset_identifier: "C".into(),
            }),
            StacksTransactionEvent::NFTBurnEvent(NFTBurnEventData {
                asset_class_identifier: format!("{}::my-nft", test_contract_id),
                sender: "S1G2081040G2081040G2081040G208105NK8P02".into(),
                asset_identifier: "".into(),
                hex_asset_identifier: "B".into(),
            }),
            StacksTransactionEvent::NFTTransferEvent(NFTTransferEventData {
                asset_class_identifier: format!("{}::my-nft", test_contract_id),
                sender: "S1G2081040G2081040G2081040G208105NK8P01".into(),
                recipient: "S1G2081040G2081040G2081040G208105NK8P03".into(),
                asset_identifier: "".into(),
                hex_asset_identifier: "A".into(),
            }),
        ]);
        let block = block_with_transactions(vec![transaction]);
        tx.send(OrchestraSupervisorMessage::ProcessStacksChainEvent(
            StacksChainEvent::ChainUpdatedWithBlock(block),
        ))
        .unwrap();

        let delay = time::Duration::from_millis(100);
        thread::sleep(delay);

        let delay = time::Duration::from_millis(100);
        thread::sleep(delay);

        tx.send(OrchestraSupervisorMessage::Exit).unwrap();

        let _res = handle.join().unwrap();

        {
            use crate::datastore::contracts::{contract_db_write, db_key, DBKey};

            let db = contract_db_write(&storage_driver, &test_contract_id.to_string());
            let other_contract_id = "S1G2081040G2081040G2081040G208105NK8P91.test";
            db.put(
                &db_key(
                    DBKey::MapEntry("my-map", "k1"),
                    &other_contract_id.to_string(),
                ),
                "junk".as_bytes(),
            )
            .unwrap();
        }

        {
            use crate::datastore::contracts::{contract_db_read, db_key, DBKey};
            use rocksdb::{Direction, IteratorMode};

            let db = contract_db_read(&storage_driver, &test_contract_id.to_string());

            let res = db
                .get(&db_key(
                    DBKey::MapEntry("my-map", "k1"),
                    &test_contract_id.to_string(),
                ))
                .unwrap()
                .unwrap();
            assert_eq!(String::from_utf8(res).unwrap(), "v1".to_string());

            let res = db
                .get(&db_key(
                    DBKey::MapEntry("my-map", "k2"),
                    &test_contract_id.to_string(),
                ))
                .unwrap()
                .unwrap();
            assert_eq!(String::from_utf8(res).unwrap(), "v4".to_string());

            let mut iter = db.iterator(IteratorMode::Start); // Always iterates forward
            println!("1");
            for (key, value) in iter {
                println!("Saw {:?}", String::from_utf8(key.to_vec()).unwrap());
            }
            iter = db.iterator(IteratorMode::End); // Always iterates backward
            println!("2");
            for (key, value) in iter {
                println!("Saw {:?}", String::from_utf8(key.to_vec()).unwrap());
            }
            iter = db.iterator(IteratorMode::From(
                b"S1G2081040G2081040G2081040G208105NK8PE5.test::var",
                Direction::Forward,
            )); // From a key in Direction::{forward,reverse}
            println!("3");
            for (key, value) in iter {
                println!("Saw {:?}", String::from_utf8(key.to_vec()).unwrap());
            }

            // You can seek with an existing Iterator instance, too
            iter = db.prefix_iterator(
                b"map::S1G2081040G2081040G2081040G208105NK8PE5.test::my-map::entry(",
            );
            // iter.set_mode(IteratorMode::From(b"S1G2081040G2081040G2081040G208105NK8PE5.test::var", Direction::Forward));
            println!("4");
            for (key, value) in iter {
                println!("Saw {:?}", String::from_utf8(key.to_vec()).unwrap());
            }
        }
    }
}
