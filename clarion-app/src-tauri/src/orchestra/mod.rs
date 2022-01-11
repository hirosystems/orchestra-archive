use std::collections::BTreeMap;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::path::PathBuf;

use clarion_lib::clarinet_lib::integrate::{DevnetOrchestrator, DevnetEvent, self};
use clarion_lib::actors::{self};
use clarion_lib::clarinet_lib::clarity_repl::clarity::analysis::contract_interface_builder::ContractInterface;

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    Message
};

pub struct GlobalState {
    pub contracts: ContractState,
}

pub struct ContractState {
    pub interface: ContractInterface,
}

pub enum BackendCommand {
    DevnetStopped,
    ChainEvent,
    Ack,
}

pub enum FrontendCommand {
    StartDevnet,
    PauseDevnet,
    GetBlock,
}

pub fn run_backend(backend_cmd_tx: Sender<BackendCommand>, frontend_cmd_rx: Receiver<FrontendCommand>) {
    use clarion_lib::actors::{ClarionSupervisorMessage};
    use clarion_lib::types::{ClarionManifest, ProjectMetadata, ContractSettings};
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
        actors::run_supervisor(supervisor_rx)
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

    let clarion_manifest = ClarionManifest {
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
    supervisor_tx.send(ClarionSupervisorMessage::RegisterManifest(clarion_manifest)).unwrap();

    let frontend_commands_supervisor_tx = supervisor_tx.clone();
    std::thread::spawn(move || {
        loop {
            let cmd = frontend_cmd_rx.recv().unwrap();

            match cmd {
                FrontendCommand::GetBlock => {

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
                    frontend_cmd_tx.send(FrontendCommand::StartDevnet).expect("Link broken");
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
                if let Ok(response) = backend_cmd_rx.recv() {
                    match response {
                        BackendCommand::DevnetStopped => {
    
                        },
                        _ => {}
                    }
                }    
            }
        }
    }
}