extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

// #[macro_use]
// extern crate lazy_static;

pub extern crate clarinet_lib;

pub mod actors;
pub mod datastore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
