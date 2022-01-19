use clarion_lib::clarinet_lib::poke::load_session_settings;
use clarion_lib::clarinet_lib::publish::Network;
use serde::{self, Deserialize, Serialize};
use serde_json::json;

use std::collections::BTreeMap;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::path::PathBuf;

use clarion_lib::clarinet_lib::integrate::{DevnetOrchestrator, DevnetEvent, self};
use clarion_lib::actors::{self};
use clarion_lib::clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::ContractInterface;
use clarion_lib::clarinet_lib::types::{BlockIdentifier, StacksBlockData, BitcoinBlockData, StacksChainEvent, BitcoinChainEvent, TransactionIdentifier, BitcoinBlockMetadata};

use clarion_lib::datastore::StorageDriver;
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AppState {
    current_block_identifier: BlockIdentifier,
    project_id: u64,
    request: AppStateRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AppStateRequest {
    Initialization,
    StateExplorer(StateExplorerState),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StateExplorerState {
    contract_identifier: String,
    field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AppStateUpdate {
    Initialization(InitializationUpdate),
    StateExplorer(StateExplorerUpdate),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InitializationUpdate {
    stacks_chain_tip: Option<StacksBlockData>,
    bitcoin_chain_tip: Option<BitcoinBlockData>, 
    contracts: Vec<ContractInterface>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateExplorerUpdate {
    stacks_chain_events: Vec<StacksChainEvent>,
    bitcoin_chain_events: Vec<BitcoinChainEvent>, 
    field_values: FieldValues, 
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FieldValues {
    Var(VarValues),
    Map(MapValues),
    Nft(NftValues),
    Ft(FtValues),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VarValues {
    value: String,
    page_size: u16,
    page_index: u64,
    events: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MapValues {
    pairs: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    page_size: u16,
    page_index: u64,
    events: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NftValues {
    tokens: Vec<((String, String), BlockIdentifier, TransactionIdentifier)>,
    page_size: u16,
    page_index: u64,
    events: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FtValues {
    balances: Vec<((String, u128), BlockIdentifier, TransactionIdentifier)>,
    page_size: u16,
    page_index: u64,
    events: Vec<u8>,
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
    PollUpdate(AppStateUpdate),
    Ack(u64),
}

pub enum FrontendCommand {
    StartDevnet,
    PauseDevnet,
    GetBlock,
    PollState(AppState),
}

pub fn run_backend(backend_cmd_tx: Sender<BackendCommand>, frontend_cmd_rx: Receiver<FrontendCommand>) {
    use clarion_lib::actors::{ClarionSupervisorMessage};
    use clarion_lib::types::{ContractsObserverConfig, ProjectMetadata, ContractSettings, ContractsObserverId};
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
    contracts.insert(test_contract_id, test_contract_settings);

    let clarion_manifest = ContractsObserverConfig {
        identifier: ContractsObserverId(1),
        project: ProjectMetadata {
            name: "test".into(),
            authors: vec![],
            homepage: "".into(),
            license: "".into(),
            description: "".into(),
        },
        lambdas: vec![],
        contracts,
    };
    supervisor_tx.send(ClarionSupervisorMessage::RegisterContractsObserver(clarion_manifest)).unwrap();

    let frontend_commands_supervisor_tx = supervisor_tx.clone();
    std::thread::spawn(move || {
        let mut ack = 1;
        loop {
            let cmd = frontend_cmd_rx.recv().unwrap();
            match cmd {
                FrontendCommand::PollState(state) => {
                    let update = match state.request {
                        AppStateRequest::Initialization => {
                            AppStateUpdate::Initialization(InitializationUpdate {
                                stacks_chain_tip: None,
                                bitcoin_chain_tip: None,
                                contracts: vec![],
                            })
                        }
                        AppStateRequest::StateExplorer(state) => {
                            AppStateUpdate::StateExplorer(StateExplorerUpdate {
                                stacks_chain_events: vec![],
                                bitcoin_chain_events: vec![],
                                field_values: FieldValues::Var(VarValues {
                                    value: "101".to_string(),
                                    page_size: 0,
                                    page_index: 0,
                                    events: vec![]
                                }),
                            })
                        }
                    };
                    backend_cmd_tx.send(BackendCommand::PollUpdate(update)).unwrap();

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


pub fn mock_backend(backend_cmd_tx: Sender<BackendCommand>, frontend_cmd_rx: Receiver<FrontendCommand>) {
    use clarion_lib::actors::{ClarionSupervisorMessage};
    use clarion_lib::types::{ContractsObserverConfig, ProjectMetadata, ContractSettings, ContractsObserverId};
    use clarion_lib::clarinet_lib::clarity_repl::clarity::types::{StandardPrincipalData, QualifiedContractIdentifier};
    use clarion_lib::clarinet_lib::types::StacksBlockMetadata;
    use std::convert::TryInto;

    let manifest_path = PathBuf::from("/Users/ludovic/Coding/clarinet/clarinet-cli/examples/counter/Clarinet.toml");

    let (session_settings, _) = load_session_settings(manifest_path, &Network::Devnet).unwrap();

    let (devnet_events_tx, devnet_events_rx) = channel::<DevnetEvent>();
    
    let (supervisor_tx, supervisor_rx) = channel();

    let handle = std::thread::spawn(|| {
        let storage_driver = StorageDriver::tmpfs();
        println!("Working dir: {:?}", storage_driver);
        actors::run_supervisor(storage_driver, supervisor_rx)
    });

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

    let test_contract_id = QualifiedContractIdentifier::new(
        StandardPrincipalData::transient(),
        "counter".try_into().unwrap(),
    );
    let test_contract_settings = ContractSettings {
        state_explorer_enabled: true,
        api_generator_enabled: vec![],
    };

    let clarion_manifest = ContractsObserverConfig {
        identifier: ContractsObserverId(1),
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

    let frontend_commands_supervisor_tx = supervisor_tx.clone();
    std::thread::spawn(move || {
        let mut ack = 1;
        loop {
            let cmd = frontend_cmd_rx.recv().unwrap();
            match cmd {
                FrontendCommand::PollState(state) => {
                    let update = match state.request {
                        AppStateRequest::Initialization => {
                            AppStateUpdate::Initialization(InitializationUpdate {
                                stacks_chain_tip: None,
                                bitcoin_chain_tip: None,
                                contracts: vec![],
                            })
                        }
                        AppStateRequest::StateExplorer(state) => {
                            AppStateUpdate::StateExplorer(StateExplorerUpdate {
                                stacks_chain_events: vec![],
                                bitcoin_chain_events: vec![],
                                field_values: FieldValues::Var(VarValues {
                                    value: "101".to_string(),
                                    page_size: 0,
                                    page_index: 0,
                                    events: vec![]
                                }),
                            })
                        }
                    };
                    backend_cmd_tx.send(BackendCommand::PollUpdate(update)).unwrap();

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

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(2),
        parent_block_identifier: block_identifier(1),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(3),
        parent_block_identifier: block_identifier(2),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(4),
        parent_block_identifier: block_identifier(3),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(5),
        parent_block_identifier: block_identifier(4),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    supervisor_tx.send(ClarionSupervisorMessage::ProcessBitcoinChainEvent(BitcoinChainEvent::ChainUpdatedWithBlock(BitcoinBlockData {
        block_identifier: block_identifier(6),
        parent_block_identifier: block_identifier(5),
        timestamp: 0,
        transactions: vec![],
        metadata: BitcoinBlockMetadata {}
    }))).unwrap();

    supervisor_tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(StacksBlockData {
        block_identifier: block_identifier(1),
        parent_block_identifier: block_identifier(0),
        timestamp: 0,
        transactions: vec![],
        metadata: StacksBlockMetadata { 
            bitcoin_anchor_block_identifier: block_identifier(1), 
            pox_cycle_index: 0, 
            pox_cycle_position: 0, 
            pox_cycle_length: 0 
        }
    }))).unwrap();

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

    supervisor_tx.send(ClarionSupervisorMessage::RegisterContractsObserver(clarion_manifest)).unwrap();



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
            headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

            Ok(response)
        };
        let mut websocket = accept_hdr(stream, callback).unwrap();

        loop {
            let msg = websocket.read_message().unwrap();
            let response_expected = match msg {
                Message::Text(msg) => {
                    println!("WS: command received");
                    if let Ok(app_state) = serde_json::from_str::<AppState>(&msg) {
                        println!("WS: Poll state command received");
                        frontend_cmd_tx.send(FrontendCommand::PollState(app_state)).expect("Link broken");
                    } else {
                        frontend_cmd_tx.send(FrontendCommand::StartDevnet).expect("Link broken");
                    }
                    true
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
                        BackendCommand::PollUpdate(update) => {
                            println!("Update {:?} received!", update);
                            websocket.write_message(Message::Text(json!(update).to_string()))
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
