#![allow(unused_imports)]

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate rocket;

pub mod observer;
pub mod indexer;
pub mod utils;