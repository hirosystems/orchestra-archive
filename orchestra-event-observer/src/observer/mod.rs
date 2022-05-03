use crate::indexer::{chains, Indexer, IndexerConfig};
use crate::utils;
use orchestra_types::{BitcoinChainEvent, ChainsCoordinatorCommand, StacksNetwork, StacksChainEvent};
use stacks_rpc_client::{PoxInfo, StacksRpc};
use rocket::config::{Config, LogLevel};
use rocket::serde::json::{json, Json, Value as JsonValue};
use rocket::serde::Deserialize;
use rocket::State;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::error::Error;
use std::iter::FromIterator;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use reqwest::Client as HttpClient;

#[derive(Deserialize)]
pub struct NewTransaction {
    pub txid: String,
    pub status: String,
    pub raw_result: String,
    pub raw_tx: String,
}

#[derive(Clone, Debug)]
pub enum Event {
    BitcoinChainEvent(BitcoinChainEvent),
    StacksChainEvent(StacksChainEvent),
}

#[derive(Clone, Debug)]
pub enum EventHandler {
    InProcess(Sender<Event>),
    WebHook(String),
}

impl EventHandler {

    async fn propagate_stacks_event(&self, stacks_event: &StacksChainEvent) {
        match self {
            EventHandler::InProcess(event_sender) => {
                let _ = event_sender.send(Event::StacksChainEvent(stacks_event.clone()));
            }
            EventHandler::WebHook(host) => {
                let path = "chain-events/stacks";
                let url = format!("{}/{}", host, path);
                let body = rocket::serde::json::serde_json::to_vec(&stacks_event).unwrap();
                let http_client = HttpClient::builder().build().expect("Unable to build http client");
                let _ = http_client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .await;
            }
        }
    }

    async fn propagate_bitcoin_event(&self, bitcoin_event: &BitcoinChainEvent) {
        match self {
            EventHandler::InProcess(event_sender) => {
                let _ = event_sender.send(Event::BitcoinChainEvent(bitcoin_event.clone()));
            }
            EventHandler::WebHook(host) => {
                let path = "chain-events/bitcoin";
                let url = format!("{}/{}", host, path);
                let body = rocket::serde::json::serde_json::to_vec(&bitcoin_event).unwrap();
                let http_client = HttpClient::builder().build().expect("Unable to build http client");
                let res = http_client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .await;
            }
        }
    }

    async fn notify_bitcoin_transaction_proxied(&self) {

    }
}

#[derive(Clone, Debug)]
pub struct StacksEventObserverConfig {
    pub normalization_enabled: bool,
    pub bitcoin_rpc_proxy_enabled: bool,
    pub event_handlers: Vec<EventHandler>,
    pub observer_port: u16,
    pub bitcoin_node_username: String,
    pub bitcoin_node_password: String,
    pub bitcoin_node_rpc_host: String,
    pub bitcoin_node_rpc_port: u16,
    pub stacks_node_rpc_host: String,
    pub stacks_node_rpc_port: u16,
}

#[derive(Deserialize, Debug)]
pub struct ContractReadonlyCall {
    pub okay: bool,
    pub result: String,
}

#[derive(Clone, Debug)]
pub enum ObserverCommand {
    PropagateBitcoinChainEvent(BitcoinChainEvent),
    PropagateStacksChainEvent(StacksChainEvent),
    NotifyBitcoinTransactionProxied,
    Terminate,
}

#[derive(Clone, Debug)]
pub enum ObserverEvent {
    Error(String),
    Fatal(String),
    Info(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// JSONRPC Request
pub struct BitcoinRPCRequest {
    /// The name of the RPC call
    pub method: String,
    /// Parameters to the RPC call
    pub params: serde_json::Value,
    /// Identifier for this Request, which should appear in the response
    pub id: serde_json::Value,
    /// jsonrpc field, MUST be "2.0"
    pub jsonrpc: serde_json::Value,
}

pub async fn start_observer(
    config: StacksEventObserverConfig,
    observer_commands_tx: Sender<ObserverCommand>,
    observer_commands_rx: Receiver<ObserverCommand>,
    observer_events_tx: Sender<ObserverEvent>,
) -> Result<(), Box<dyn Error>> {

    let indexer = Indexer::new(IndexerConfig {
        stacks_node_rpc_url: format!(
            "http://{}:{}",
            config.stacks_node_rpc_host,
            config.stacks_node_rpc_port
        ),
        bitcoin_node_rpc_url: format!(
            "http://{}:{}",
            config.bitcoin_node_rpc_host,
            config.bitcoin_node_rpc_port
        ),
        bitcoin_node_rpc_username: config.bitcoin_node_username.clone(),
        bitcoin_node_rpc_password: config.bitcoin_node_password.clone(),
    });

    let port = config.observer_port;

    let config_mutex = Arc::new(Mutex::new(config.clone()));
    let indexer_rw_lock = Arc::new(RwLock::new(indexer));

    let background_job_tx_mutex = Arc::new(Mutex::new(observer_commands_tx.clone()));

    let rocket_config = Config {
        port: port,
        workers: 4,
        address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        keep_alive: 5,
        temp_dir: std::env::temp_dir(),
        log_level: LogLevel::Debug,
        ..Config::default()
    };

    let mut routes = routes![
        handle_ping,
        handle_new_bitcoin_block,
        handle_new_stacks_block,
        handle_new_microblocks,
        handle_new_mempool_tx,
        handle_drop_mempool_tx,
    ];

    if config.bitcoin_rpc_proxy_enabled {
        routes.append(&mut routes![handle_bitcoin_rpc_call]);
    }

    let _ = std::thread::spawn(move || {
        let future = rocket::custom(rocket_config)
            .manage(indexer_rw_lock)
            .manage(config_mutex)
            .manage(background_job_tx_mutex)
            .mount(
                "/",
                routes,
            )
            .launch();
        let rt = utils::create_basic_runtime();
        rt.block_on(future).expect("Unable to spawn event observer");
    });

    // This loop is used for handling background jobs, emitted by HTTP calls.
    let stop_miner = Arc::new(AtomicBool::new(false));

    loop {
        let command = match observer_commands_rx.recv() {
            Ok(cmd) => cmd,
            Err(e) => {
                let _ = observer_events_tx.send(ObserverEvent::Error(format!("Chanel error: {:?}", e)));
                continue;
            }
        };
        match command {
            ObserverCommand::Terminate => {
                let _ = observer_events_tx.send(ObserverEvent::Info("Terminating event observer".into()));
                break;
            }
            ObserverCommand::PropagateBitcoinChainEvent(event) => {
                for event_handler in config.event_handlers.iter() {
                    event_handler.propagate_bitcoin_event(&event).await;
                }
            }
            ObserverCommand::PropagateStacksChainEvent(event) => {
                for event_handler in config.event_handlers.iter() {
                    event_handler.propagate_stacks_event(&event).await;
                }
            }
            ObserverCommand::NotifyBitcoinTransactionProxied => {
                for event_handler in config.event_handlers.iter() {
                    event_handler.notify_bitcoin_transaction_proxied().await;
                }
            }
        }
    }
    Ok(())
}

#[get("/ping", format = "application/json")]
pub fn handle_ping() -> Json<JsonValue> {
    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post("/new_burn_block", format = "json", data = "<marshalled_block>")]
pub fn handle_new_bitcoin_block(
    indexer_rw_lock: &State<Arc<RwLock<Indexer>>>,
    marshalled_block: Json<JsonValue>,
    background_job_tx: &State<Arc<Mutex<Sender<ObserverCommand>>>>,
) -> Json<JsonValue> {

    // Standardize the structure of the block, and identify the
    // kind of update that this new block would imply, taking
    // into account the last 7 blocks.
    let chain_update = match indexer_rw_lock.inner().write() {
        Ok(mut indexer) => indexer.handle_bitcoin_block(marshalled_block.into_inner()),
        _ => {
            return Json(json!({
                "status": 200,
                "result": "Ok",
            }))
        }
    };

    let background_job_tx = background_job_tx.inner();
    match background_job_tx.lock() {
        Ok(tx) => {
            let _ = tx.send(ObserverCommand::PropagateBitcoinChainEvent(chain_update));
        }
        _ => {}
    };

    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post("/new_block", format = "application/json", data = "<marshalled_block>")]
pub fn handle_new_stacks_block(
    indexer_rw_lock: &State<Arc<RwLock<Indexer>>>,
    marshalled_block: Json<JsonValue>,
    background_job_tx: &State<Arc<Mutex<Sender<ObserverCommand>>>>,
) -> Json<JsonValue> {
    // Standardize the structure of the block, and identify the
    // kind of update that this new block would imply, taking
    // into account the last 7 blocks.
    let (pox_info, chain_event) = match indexer_rw_lock.inner().write() {
        Ok(mut indexer) => {
            let pox_info = indexer.get_pox_info();
            let chain_event = indexer.handle_stacks_block(marshalled_block.into_inner());
            (pox_info, chain_event)
        }
        _ => {
            return Json(json!({
                "status": 200,
                "result": "Ok",
            }))
        }
    };

    let background_job_tx = background_job_tx.inner();
    match background_job_tx.lock() {
        Ok(tx) => {
            let _ = tx.send(ObserverCommand::PropagateStacksChainEvent(chain_event));
        }
        _ => {}
    };

    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post(
    "/new_microblocks",
    format = "application/json",
    data = "<marshalled_microblock>"
)]
pub fn handle_new_microblocks(
    indexer_rw_lock: &State<Arc<RwLock<Indexer>>>,
    marshalled_microblock: Json<JsonValue>,
    background_job_tx: &State<Arc<Mutex<Sender<ObserverCommand>>>>,
) -> Json<JsonValue> {

    // Standardize the structure of the microblock, and identify the
    // kind of update that this new microblock would imply
    let chain_event = match indexer_rw_lock.inner().write() {
        Ok(mut indexer) => {
            let chain_event = indexer.handle_stacks_microblock(marshalled_microblock.into_inner());
            chain_event
        }
        _ => {
            return Json(json!({
                "status": 200,
                "result": "Ok",
            }))
        }
    };

    let background_job_tx = background_job_tx.inner();
    match background_job_tx.lock() {
        Ok(tx) => {
            let _ = tx.send(ObserverCommand::PropagateStacksChainEvent(chain_event));
        }
        _ => {}
    };

    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post("/new_mempool_tx", format = "application/json", data = "<raw_txs>")]
pub fn handle_new_mempool_tx(
    raw_txs: Json<Vec<String>>,
    background_job_tx: &State<Arc<Mutex<Sender<ObserverCommand>>>>,
) -> Json<JsonValue> {
    let decoded_transactions = raw_txs
        .iter()
        .map(|t| {
            let (txid, ..) =
                chains::stacks::get_tx_description(t).expect("unable to parse transaction");
            txid
        })
        .collect::<Vec<String>>();

    // if let Ok(tx_sender) = devnet_events_tx.lock() {
    //     for tx in decoded_transactions.into_iter() {
    //         let _ = tx_sender.send(DevnetEvent::MempoolAdmission(MempoolAdmissionData { tx }));
    //     }
    // }

    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post("/drop_mempool_tx", format = "application/json")]
pub fn handle_drop_mempool_tx() -> Json<JsonValue> {
    Json(json!({
        "status": 200,
        "result": "Ok",
    }))
}

#[post("/", format = "application/json", data = "<bitcoin_rpc_call>")]
pub async fn handle_bitcoin_rpc_call(
    config: &State<Arc<Mutex<StacksEventObserverConfig>>>,
    bitcoin_rpc_call: Json<BitcoinRPCRequest>,
    background_job_tx: &State<Arc<Mutex<Sender<ObserverCommand>>>>,
) -> Json<JsonValue> {
    use base64::encode;
    use reqwest::Client;

    let bitcoin_rpc_call = bitcoin_rpc_call.into_inner().clone();
    let method = bitcoin_rpc_call.method.clone();
    let body = rocket::serde::json::serde_json::to_vec(&bitcoin_rpc_call).unwrap();

    let builder = match config.inner().lock() {
        Ok(config) => {
            let token = encode(format!(
                "{}:{}",
                config.bitcoin_node_username,
                config.bitcoin_node_password
            ));

            let client = Client::new();
            client
                .post(format!(
                    "http://localhost:{}/",
                    config.bitcoin_node_rpc_port
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Basic {}", token))
        }
        _ => unreachable!(),
    };

    if method == "sendrawtransaction" {
        let background_job_tx = background_job_tx.inner();
        match background_job_tx.lock() {
            Ok(tx) => {
                // let _ = tx.send(ObserverCommand::PropagateStacksChainEvent(chain_event));
            }
            _ => {}
        };
    
        // if let Ok(background_job_tx) = background_job_tx_mutex.lock() {
        //     let _ = background_job_tx.send(ChainsCoordinatorCommand::BitcoinOpSent);
        // }
    }

    let res = builder.body(body).send().await.unwrap();

    Json(res.json().await.unwrap())
}
