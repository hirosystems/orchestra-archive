use clarion_lib::clarinet_lib::clarity_repl::repl::{Session, SessionSettings};
use clarion_lib::clarinet_lib::poke::load_session_settings;
use clarion_lib::clarinet_lib::publish::Network;
use serde::{self, Deserialize, Serialize};
use serde_json::json;

use std::collections::{BTreeMap, HashSet};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::path::PathBuf;
use std::thread;
use std::time;

use clarion_lib::clarinet_lib::integrate::{DevnetOrchestrator, DevnetEvent, self};
use clarion_lib::actors::{self};
use clarion_lib::clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::{ContractInterface, build_contract_interface};
use clarion_lib::clarinet_lib::types::{StacksTransactionReceipt, BlockIdentifier, StacksBlockData, BitcoinBlockData, StacksChainEvent, BitcoinChainEvent, TransactionIdentifier, BitcoinBlockMetadata, StacksTransactionData, StacksTransactionMetadata, StacksTransactionKind, StacksContractDeploymentData};
use clarion_lib::clarinet_lib::types::events::{StacksTransactionEvent, DataVarSetEventData, DataMapInsertEventData, DataMapUpdateEventData, DataMapDeleteEventData};
use clarion_lib::types::{ProtocolObserverConfig, FieldValues, FieldValuesRequest, FieldValuesResponse, VarValues, Contract, ProtocolObserverId};

use clarion_lib::datastore::StorageDriver;
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PollState {
    protocol_id: u64,
    request: NetworkRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum NetworkRequest {
    StateExplorerInitialization(StateExplorerInitialization),
    StateExplorerWatch(StateExplorerWatch),
    StateExplorerSync(StateExplorerSync),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StateExplorerInitialization {
    manifest_path: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StateExplorerSync {
    stacks_block_identifier: Option<BlockIdentifier>,
    bitcoin_block_identifier: Option<BlockIdentifier>,
    expected_contracts_identifiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StateExplorerWatch {
    stacks_block_identifier: BlockIdentifier,
    bitcoin_block_identifier: BlockIdentifier,
    target: StateExplorerWatchTarget,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StateExplorerWatchTarget {
    ContractField(ContractFieldData),
    Wallet(WalletData)
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContractFieldData {
    contract_identifier: String,
    field_name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WalletData {
    address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NetworkResponse {
    StateExplorerInitialization(StateExplorerInitializationUpdate),
    StateExplorerSync(StateExplorerSyncUpdate),
    StateExplorerWatch(StateExplorerWatchUpdate),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateExplorerInitializationUpdate {
    contracts: Vec<Contract>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateExplorerSyncUpdate {
    stacks_chain_tip: Option<StacksBlockData>,
    bitcoin_chain_tip: Option<BitcoinBlockData>, 
    contracts: Vec<Contract>,
    expected_contracts_identifiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateExplorerWatchUpdate {
    stacks_chain_blocks: Vec<StacksBlockData>,
    bitcoin_chain_blocks: Vec<BitcoinBlockData>, 
    contract_identifier: String,
    field_name: String,
    field_values: FieldValues, 
}

pub struct GlobalState {
    pub contracts: ContractState,
}

pub struct ContractState {
    pub interface: ContractInterface,
}

pub enum BackendCommand {
    DevnetStopped,
    ChainEvent,
    Poll(NetworkResponse),
    Ack(u64),
}

pub enum FrontendCommand {
    StartDevnet,
    PauseDevnet,
    GetBlock,
    PollState(PollState),
}

pub fn run_backend(backend_cmd_tx: Sender<BackendCommand>, frontend_cmd_rx: Receiver<FrontendCommand>) {
    use clarion_lib::actors::{ClarionSupervisorMessage};
    use clarion_lib::types::{ProtocolObserverConfig, ProjectMetadata, ContractSettings, ProtocolObserverId};
    use clarion_lib::clarinet_lib::clarity_repl::clarity::types::{StandardPrincipalData, QualifiedContractIdentifier};
    use std::convert::TryInto;

    let (log_tx, log_rx) = channel();
    let manifest_path = PathBuf::from("/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml");
    let devnet = DevnetOrchestrator::new(manifest_path, None);

    let (devnet_events_rx, terminator_tx) =
    match integrate::run_devnet(devnet, Some(log_tx), false) {
        Ok((Some(devnet_events_rx), Some(terminator_tx))) => {
            (devnet_events_rx, terminator_tx)
        }
        _ => std::process::exit(1),
    };

    let (supervisor_tx, supervisor_rx) = channel();

    let handle = std::thread::spawn(|| {
        let storage_driver = StorageDriver::tmpfs();
        println!("Working dir: {:?}", storage_driver);
        actors::run_supervisor(storage_driver, supervisor_rx)
    });

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

    let clarion_manifest = ProtocolObserverConfig {
        identifier: ProtocolObserverId(1),
        project: ProjectMetadata {
            name: "my-project".into(),
            authors: vec![],
            homepage: "".into(),
            license: "".into(),
            description: "".into(),
        },
        lambdas: vec![],
        contracts,
    };
    supervisor_tx.send(ClarionSupervisorMessage::RegisterProtocolObserver(clarion_manifest)).unwrap();

    let frontend_commands_supervisor_tx = supervisor_tx.clone();
    std::thread::spawn(move || {
        let mut ack = 1;
        loop {
            let cmd = frontend_cmd_rx.recv().unwrap();
            match cmd {
                FrontendCommand::PollState(state) => {
                    let update = match state.request {
                        NetworkRequest::StateExplorerInitialization(state) => {
                            NetworkResponse::StateExplorerInitialization(StateExplorerInitializationUpdate {
                                contracts: vec![]
                            })
                        }
                        NetworkRequest::StateExplorerSync(state) => {
                            NetworkResponse::StateExplorerSync(StateExplorerSyncUpdate {
                                stacks_chain_tip: None,
                                bitcoin_chain_tip: None,
                                contracts: vec![],
                                expected_contracts_identifiers: vec![]
                            })
                        }
                        NetworkRequest::StateExplorerWatch(watch_state) => {
                            match watch_state.target {
                                StateExplorerWatchTarget::ContractField(field) => {
                                    // Get the latest blocks
                                    // Get the latest values
                                    let (tx, rx) = channel();
                                    frontend_commands_supervisor_tx.send(ClarionSupervisorMessage::GetFieldValues(FieldValuesRequest {
                                        protocol_id: state.protocol_id,
                                        tx,
                                        contract_identifier: field.contract_identifier.clone(),
                                        field_name: field.field_name.clone(),
                                    }));
                                    let response = rx.recv().unwrap();

                                    NetworkResponse::StateExplorerWatch(StateExplorerWatchUpdate {
                                        stacks_chain_blocks: vec![],
                                        bitcoin_chain_blocks: vec![],
                                        contract_identifier: response.contract_identifier.clone(),
                                        field_name: response.field_name.clone(),
                                        field_values: response.values.clone(),
                                    })
                                }
                                StateExplorerWatchTarget::Wallet(wallet) => {
                                    unreachable!()   
                                }
                            }
                        }
                    };
                    backend_cmd_tx.send(BackendCommand::Poll(update)).unwrap();

                }
                FrontendCommand::GetBlock => {
                    ack += 1;
                    backend_cmd_tx.send(BackendCommand::Ack(ack)).unwrap();
                }
                _ => {

                }
            }
        }
    });

    loop {
        let event = devnet_events_rx.recv().unwrap();
        if let DevnetEvent::BitcoinChainEvent(event) = event {
            supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(event)).unwrap();
        } else if let DevnetEvent::StacksChainEvent(event) = event {
            supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(event)).unwrap();
        }
    }
}

pub fn config_and_interface_from_clarinet_manifest_path(manifest_path: &str) -> (ProtocolObserverConfig, Vec<Contract>) {
    use clarion_lib::types::{ProjectMetadata, ContractSettings, ProtocolObserverId};
    use clarion_lib::clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;

    let manifest_path = PathBuf::from(manifest_path);

    let (session_settings, _) = load_session_settings(manifest_path, &Network::Devnet).unwrap();
    
    let mut session = Session::new(session_settings.clone());
    let analysis = match session.start() {
        Ok((res, analysis)) => analysis,
        Err(e) => panic!(),
    };

    let interfaces = analysis.iter().map(|(a, id, _)| 
        Contract { 
            contract_identifier: a.contract_identifier.to_string(), 
            interface: build_contract_interface(a)
        }).collect::<_>();

    println!("{:?}", interfaces);

    let mut observed_contracts = BTreeMap::new();
    for contract in session_settings.initial_contracts.iter() {
        let contract_id = QualifiedContractIdentifier::parse(
            &format!("{}.{}", session_settings.initial_deployer.clone().unwrap().address, contract.name.clone().unwrap())
        ).unwrap();

        observed_contracts.insert(contract_id, ContractSettings {
            state_explorer_enabled: true,
            api_generator_enabled: vec![],
        });
    }

    let clarion_manifest = ProtocolObserverConfig {
        identifier: ProtocolObserverId(1),
        project: ProjectMetadata {
            name: "counter".into(),
            authors: vec![],
            homepage: "".into(),
            license: "".into(),
            description: "".into(),
        },
        lambdas: vec![],
        contracts: observed_contracts,
    };
    (clarion_manifest, interfaces)
}

pub fn config_from_clarinet_manifest_path(manifest_path: &str) -> (ProtocolObserverConfig, SessionSettings) {
    use clarion_lib::types::{ProjectMetadata, ContractSettings, ProtocolObserverId};
    use clarion_lib::clarinet_lib::clarity_repl::clarity::types::QualifiedContractIdentifier;

    let manifest_path = PathBuf::from(manifest_path);

    let (session_settings, _) = load_session_settings(manifest_path, &Network::Devnet).unwrap();
    
    let mut observed_contracts = BTreeMap::new();
    for contract in session_settings.initial_contracts.iter() {
        let contract_id = QualifiedContractIdentifier::parse(
            &format!("{}.{}", session_settings.initial_deployer.clone().unwrap().address, contract.name.clone().unwrap())
        ).unwrap();

        observed_contracts.insert(contract_id, ContractSettings {
            state_explorer_enabled: true,
            api_generator_enabled: vec![],
        });
    }

    let clarion_manifest = ProtocolObserverConfig {
        identifier: ProtocolObserverId(1),
        project: ProjectMetadata {
            name: "counter".into(),
            authors: vec![],
            homepage: "".into(),
            license: "".into(),
            description: "".into(),
        },
        lambdas: vec![],
        contracts: observed_contracts,
    };
    (clarion_manifest, session_settings)
}


pub fn mock_backend(backend_cmd_tx: Sender<BackendCommand>, frontend_cmd_rx: Receiver<FrontendCommand>) {
    use clarion_lib::actors::{ClarionSupervisorMessage};
    use clarion_lib::clarinet_lib::types::StacksBlockMetadata;

    let (supervisor_tx, supervisor_rx) = channel();

    let handle = std::thread::spawn(|| {
        let storage_driver = StorageDriver::tmpfs();
        println!("Working dir: {:?}", storage_driver);
        actors::run_supervisor(storage_driver, supervisor_rx)
    });

    let frontend_commands_supervisor_tx = supervisor_tx.clone();
    std::thread::spawn(move || {
        let mut ack = 1;
        loop {
            let cmd = frontend_cmd_rx.recv().unwrap();
            match cmd {
                FrontendCommand::PollState(state) => {
                    let update = match state.request {
                        NetworkRequest::StateExplorerInitialization(state_init) => {
                            
                            let (config, settings) = config_from_clarinet_manifest_path(&state_init.manifest_path);

                            let mut transactions = vec![];
                            for contract in settings.initial_contracts.iter() {
                                transactions.push(StacksTransactionData {
                                    transaction_identifier: TransactionIdentifier { hash: "0".into() },
                                    operations: vec![],
                                    metadata: StacksTransactionMetadata {
                                        success: true,
                                        description: "".into(),
                                        sponsor: None,
                                        raw_tx: "".into(),
                                        result: "(ok true)".into(),
                                        sender: contract.deployer.clone().unwrap(),
                                        fee: 1,
                                        kind: StacksTransactionKind::ContractDeployment(StacksContractDeploymentData {
                                            contract_identifier: format!("{}.{}", contract.deployer.clone().unwrap(), contract.name.clone().unwrap()),
                                            code: contract.code.clone(),
                                        }),
                                        execution_cost: None,
                                        receipt: StacksTransactionReceipt {
                                            mutated_contracts_radius: HashSet::new(),
                                            mutated_assets_radius: HashSet::new(),
                                            events: vec![],
                                        }
                                    }
                                });
                            }
                            // Build a temporary block that the registration can rely on for the ProtocolRegistration.
                            // Local only
                            frontend_commands_supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(StacksBlockData {
                                block_identifier: block_identifier(0),
                                parent_block_identifier: block_identifier(0),
                                timestamp: 0,
                                transactions,
                                metadata: StacksBlockMetadata { 
                                    bitcoin_anchor_block_identifier: block_identifier(1), 
                                    pox_cycle_index: 0, 
                                    pox_cycle_position: 0, 
                                    pox_cycle_length: 0 
                                }
                            }))).unwrap();

                            frontend_commands_supervisor_tx.send(ClarionSupervisorMessage::RegisterProtocolObserver(config)).unwrap();
                            
                            let (tx, rx) = channel();
                            frontend_commands_supervisor_tx.send(ClarionSupervisorMessage::GetProtocolInterfaces(ProtocolObserverId(state.protocol_id), tx)).unwrap();
                            let response = rx.recv().unwrap();

                            NetworkResponse::StateExplorerInitialization(StateExplorerInitializationUpdate {
                                contracts: response.contracts,
                            })
                        }
                        NetworkRequest::StateExplorerSync(state) => {
                            let bitcoin_chain_tip = get_bitcoin_chain_tip(state.bitcoin_block_identifier.as_ref());
                            let stacks_chain_tip = get_stacks_chain_tip(state.bitcoin_block_identifier.as_ref());
        
                            NetworkResponse::StateExplorerSync(StateExplorerSyncUpdate {
                                stacks_chain_tip: None,
                                bitcoin_chain_tip: None,
                                contracts: vec![],
                                expected_contracts_identifiers: vec![]
                            })
                        }
                        NetworkRequest::StateExplorerWatch(watch_state) => {
                            match watch_state.target {
                                StateExplorerWatchTarget::ContractField(field) => {
                                    // Get the latest blocks
                                    // Get the latest values
                                    let (tx, rx) = channel();
                                    frontend_commands_supervisor_tx.send(ClarionSupervisorMessage::GetFieldValues(FieldValuesRequest {
                                        protocol_id: state.protocol_id,
                                        tx,
                                        contract_identifier: field.contract_identifier.clone(),
                                        field_name: field.field_name.clone(),
                                    }));
                                    let response = rx.recv().unwrap();

                                    NetworkResponse::StateExplorerWatch(StateExplorerWatchUpdate {
                                        stacks_chain_blocks: vec![],
                                        bitcoin_chain_blocks: vec![],
                                        contract_identifier: response.contract_identifier.clone(),
                                        field_name: response.field_name.clone(),
                                        field_values: response.values.clone(),
                                    })
                                }
                                StateExplorerWatchTarget::Wallet(wallet) => {
                                    unreachable!()   
                                }
                            }
                        }
                    };
                    backend_cmd_tx.send(BackendCommand::Poll(update)).unwrap();
                }
                FrontendCommand::GetBlock => {
                    ack += 1;
                    backend_cmd_tx.send(BackendCommand::Ack(ack)).unwrap();
                }
                _ => {

                }
            }
        }
    });

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(1),
        parent_block_identifier: block_identifier(0),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    let delay = time::Duration::from_millis(10000);
    thread::sleep(delay);

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(2),
        parent_block_identifier: block_identifier(1),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    let delay = time::Duration::from_millis(10000);
    thread::sleep(delay);

    let counter_contract = "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.counter";
    let mut mutated_contracts_radius = HashSet::new();
    mutated_contracts_radius.insert(counter_contract.into());
    let mut transactions = vec![];
    transactions.push(StacksTransactionData {
        transaction_identifier: TransactionIdentifier { hash: "0".into() },
        operations: vec![],
        metadata: StacksTransactionMetadata {
            success: true,
            description: "".into(),
            sponsor: None,
            raw_tx: "".into(),
            result: "(ok true)".into(),
            sender: "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM".into(),
            fee: 1,
            kind: StacksTransactionKind::ContractCall,
            execution_cost: None,
            receipt: StacksTransactionReceipt {
                mutated_contracts_radius: mutated_contracts_radius,
                mutated_assets_radius: HashSet::new(),
                events: vec![
                    StacksTransactionEvent::DataVarSetEvent(DataVarSetEventData {
                        contract_identifier: counter_contract.into(),
                        var: "counter".into(),
                        new_value: "".into(),
                        hex_new_value: "0100000000000000000000000000000065".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "simple-kv".into(),
                        inserted_key: "".into(),
                        inserted_value: "".into(),
                        hex_inserted_key: "0100000000000000000000000000000001".into(),
                        hex_inserted_value: "01000000000000000000000000000f4240".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "simple-kv".into(),
                        inserted_key: "".into(),
                        inserted_value: "".into(),
                        hex_inserted_key: "0100000000000000000000000000000002".into(),
                        hex_inserted_value: "01000000000000000000000000001e8480".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "simple-kv".into(),
                        inserted_key: "".into(),
                        inserted_value: "".into(),
                        hex_inserted_key: "0100000000000000000000000000000003".into(),
                        hex_inserted_value: "01000000000000000000000000001e8480".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "simple-kv".into(),
                        inserted_key: "".into(),
                        inserted_value: "".into(),
                        hex_inserted_key: "0100000000000000000000000000000004".into(),
                        hex_inserted_value: "01000000000000000000000000001e8480".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "multi-kv".into(),
                        inserted_key: "(tuple (key1 u11) (key2 u12))".into(),
                        inserted_value: "(tuple (value1 u1001) (value2 u1002) (value3 u1003))".into(),
                        hex_inserted_key: "0c00000002046b657931010000000000000000000000000000000b046b657932010000000000000000000000000000000c".into(),
                        hex_inserted_value: "0c000000030676616c75653101000000000000000000000000000003e90676616c75653201000000000000000000000000000003ea0676616c75653301000000000000000000000000000003eb".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "multi-kv".into(),
                        inserted_key: "(tuple (key1 u21) (key2 u22))".into(),
                        inserted_value: "(tuple (value1 u2001) (value2 u2002) (value3 u2003))".into(),
                        hex_inserted_key: "0c00000002046b6579310100000000000000000000000000000015046b6579320100000000000000000000000000000016".into(),
                        hex_inserted_value: "0c000000030676616c75653101000000000000000000000000000007d10676616c75653201000000000000000000000000000007d20676616c75653301000000000000000000000000000007d3".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "multi-kv".into(),
                        inserted_key: "(tuple (key1 u31) (key2 u32))".into(),
                        inserted_value: "(tuple (value1 u3001) (value2 u3002) (value3 u3003))".into(),
                        hex_inserted_key: "0c00000002046b657931010000000000000000000000000000001f046b6579320100000000000000000000000000000020".into(),
                        hex_inserted_value: "0c000000030676616c7565310100000000000000000000000000000bb90676616c7565320100000000000000000000000000000bba0676616c7565330100000000000000000000000000000bbb".into(),
                    }),
                    StacksTransactionEvent::DataMapInsertEvent(DataMapInsertEventData {
                        contract_identifier: counter_contract.into(),
                        map: "multi-kv".into(),
                        inserted_key: "(tuple (key1 u41) (key2 u42))".into(),
                        inserted_value: "(tuple (value1 u4001) (value2 u4002) (value3 u4003))".into(),
                        hex_inserted_key: "0c00000002046b6579310100000000000000000000000000000029046b657932010000000000000000000000000000002a".into(),
                        hex_inserted_value: "0c000000030676616c7565310100000000000000000000000000000fa10676616c7565320100000000000000000000000000000fa20676616c7565330100000000000000000000000000000fa3".into(),
                    })
                ],
            }
        }
    });

    // DataVarSetEvent(DataVarSetEventData),
    // DataMapInsertEvent(DataMapInsertEventData),
    // DataMapUpdateEvent(DataMapUpdateEventData),
    // DataMapDeleteEvent(DataMapDeleteEventData),


    supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(StacksBlockData {
        block_identifier: block_identifier(1),
        parent_block_identifier: block_identifier(0),
        timestamp: 0,
        transactions: transactions,
        metadata: StacksBlockMetadata { 
            bitcoin_anchor_block_identifier: block_identifier(1), 
            pox_cycle_index: 0, 
            pox_cycle_position: 0, 
            pox_cycle_length: 0 
        }
    }))).unwrap();

    let delay = time::Duration::from_millis(10000);
    thread::sleep(delay);

    supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(StacksBlockData {
        block_identifier: block_identifier(2),
        parent_block_identifier: block_identifier(1),
        timestamp: 0,
        transactions: vec![],
        metadata: StacksBlockMetadata { 
            bitcoin_anchor_block_identifier: block_identifier(1), 
            pox_cycle_index: 0, 
            pox_cycle_position: 0, 
            pox_cycle_length: 0 
        }
    }))).unwrap();

    let delay = time::Duration::from_millis(10000);
    thread::sleep(delay);

    supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(StacksBlockData {
        block_identifier: block_identifier(3),
        parent_block_identifier: block_identifier(2),
        timestamp: 0,
        transactions: vec![],
        metadata: StacksBlockMetadata { 
            bitcoin_anchor_block_identifier: block_identifier(1), 
            pox_cycle_index: 0, 
            pox_cycle_position: 0, 
            pox_cycle_length: 0 
        }
    }))).unwrap();
}

pub fn run_frontend(frontend_cmd_tx: Sender<FrontendCommand>, backend_cmd_rx: Receiver<BackendCommand>) {

    let server = TcpListener::bind("127.0.0.1:2404").unwrap();
    if let Some(Ok(stream)) = server.incoming().next() {
        let callback = |req: &Request, mut response: Response| {
            println!("Received a new ws handshake");
            println!("The request's path is: {}", req.uri().path());
            println!("The request's headers are:");
            for (ref header, _value) in req.headers() {
                println!("* {}", header);
            }

            // Let's add an additional header to our response to the client.
            let headers = response.headers_mut();
            headers.append("MyCustomHeader", ":)".parse().unwrap());

            Ok(response)
        };
        let mut websocket = accept_hdr(stream, callback).unwrap();
        let mut initialized = false;

        loop {
            let msg = websocket.read_message().unwrap();
            let response_expected = match msg {
                Message::Text(msg) => {
                    // let poll_state = PollState {
                    //     protocol_id: 0,
                    //     request: NetworkRequest::StateExplorerInitialization(StateExplorerInitialization {
                    //         manifest_path: "/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml".into()
                    //     })
                    // };
                    let poll_state = PollState {
                        protocol_id: 1,
                        request: NetworkRequest::StateExplorerWatch(StateExplorerWatch {
                            stacks_block_identifier: BlockIdentifier { index: 1, hash: "1".to_string() },
                            bitcoin_block_identifier: BlockIdentifier { index: 1, hash: "1".to_string() },
                            target: StateExplorerWatchTarget::ContractField(ContractFieldData {
                                contract_identifier: "1".to_string(),
                                field_name: "var".to_string(),
                            })
                        })
                    };
                    println!("WS: command received: \n{}\n{}", msg, json!(poll_state).to_string());
                    let response_expected = if let Ok(app_state) = serde_json::from_str::<PollState>(&msg) {
                        if let NetworkRequest::StateExplorerInitialization(_) = app_state.request {
                            if !initialized {
                                println!("WS: NetworkCommand received {:?}", app_state);
                                frontend_cmd_tx.send(FrontendCommand::PollState(app_state)).expect("Link broken");
                                initialized = true;
                                true
                            } else {
                                false
                            }    
                        } else {
                            frontend_cmd_tx.send(FrontendCommand::PollState(app_state)).expect("Link broken");
                            true
                        }
                    } else {
                        false
                    };
                    response_expected
                },
                Message::Binary(bytes) => {
                    true
                },
                Message::Ping(bytes) => {
                    true
                },
                Message::Pong(bytes) => {
                    true
                },
                Message::Close(close_cmd) => {
                    true
                },                
            };

            if response_expected {
                println!("Waiting for response");
                if let Ok(response) = backend_cmd_rx.recv() {
                    match response {
                        BackendCommand::Ack(ack) => {
                            println!("ACK {} received!", ack);
                            websocket.write_message(Message::Text(json!({
                                "msg": format!("Ack {}", ack)
                            }).to_string())).expect("Link broken");
                        },
                        BackendCommand::Poll(update) => {
                            println!("Sending {} received!", json!({
                                "update": update
                            }));
                            websocket.write_message(Message::Text(json!({
                                "update": update
                            }).to_string()))
                                .expect("Link broken");
                        },

                        BackendCommand::DevnetStopped => {
    
                        },
                        _ => {}
                    }
                }    
            }
        }
    }
}

fn block_identifier(i: u64) -> BlockIdentifier {
    BlockIdentifier {
        index: i,
        hash: format!("{}", i),
    }
}

fn get_bitcoin_chain_tip(known_tip: Option<&BlockIdentifier>) -> Option<BitcoinBlockData> {
    None
}

fn get_stacks_chain_tip(known_tip: Option<&BlockIdentifier>) -> Option<StacksBlockData> {
    None
}