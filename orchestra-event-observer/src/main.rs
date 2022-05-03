#![allow(unused_imports)]

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate rocket;

mod observer;
mod indexer;
mod utils;

use std::sync::mpsc::{channel, Receiver, Sender};
use observer::{StacksEventObserverConfig, EventHandler};

fn main() {
    let (command_tx, command_rx) = channel();
    let (event_tx, event_rx) = channel();
    let config = StacksEventObserverConfig {
        normalization_enabled: true,
        bitcoin_rpc_proxy_enabled: false,
        event_handlers: vec![EventHandler::WebHook("http://0.0.0.0:19999".into())],
        observer_port: 9999,
        bitcoin_node_username: "devnet".into(),
        bitcoin_node_password: "devnet".into(),
        bitcoin_node_rpc_host: "0.0.0.0".into(),
        bitcoin_node_rpc_port: 18443,
        stacks_node_rpc_host: "0.0.0.0".into(),
        stacks_node_rpc_port: 20443,
    };
    let future = observer::start_observer(config, command_tx, command_rx, event_tx);
    let rt = utils::create_basic_runtime();
    rt.block_on(future).expect("Unable to spawn event observer");
}
