mod supervisor;
mod contract_processor;
mod contracts_observer;
mod block_store_manager;

pub use supervisor::{ClarionSupervisor, ClarionSupervisorMessage};
pub use contract_processor::{ContractProcessor, ContractProcessorMessage};
pub use contracts_observer::{ContractsObserver, ContractsObserverMessage};
pub use block_store_manager::{BlockStoreManager, BlockStoreManagerMessage};

use std::sync::mpsc::{Receiver};
use std::sync::Arc;
use kompact::prelude::*;

use crate::datastore::StorageDriver;

pub fn run_supervisor(
    storage_driver: StorageDriver,
    supervisor_cmd_rx: Receiver<ClarionSupervisorMessage>,
) -> Result<(), String> {
    match block_on(do_run_supervisor(
        storage_driver,
        supervisor_cmd_rx,
    )) {
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
    supervisor_cmd_rx: Receiver<ClarionSupervisorMessage>,
) -> Result<(), String> {
    let system = KompactConfig::default().build().expect("system");
    
    let supervisor: Arc<Component<ClarionSupervisor>> = system.create(|| ClarionSupervisor::new(storage_driver) );
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

    use std::time::SystemTime;
    use opentelemetry::{KeyValue};
    use opentelemetry::trace::{StatusCode, Span, SpanContext};
    use clarinet_lib::types::{BlockIdentifier, StacksBlockMetadata, StacksBlockData, StacksTransactionData, TransactionIdentifier, StacksTransactionMetadata, StacksTransactionReceipt};
    use std::collections::HashSet;
    use crate::datastore::StorageDriver;
    use crate::actors::{ClarionSupervisorMessage};

    #[derive(Debug)]
    struct MockedSpan {
        context: SpanContext
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
        ) {}
        fn span_context(&self) -> &SpanContext {
            return &self.context
        }
        fn is_recording(&self) -> bool { true }
        fn set_attribute(&mut self, _attribute: KeyValue) {}
        fn set_status(&mut self, _code: StatusCode, _message: String) {}
        fn update_name(&mut self, _new_name: String) {}
        fn end(&mut self) {}
        fn end_with_timestamp(&mut self, _timestamp: SystemTime) {}
    }


    fn transaction_impacting_contract_id(contract_id: String, success: bool) -> StacksTransactionData {
        let mut mutated_contracts_radius = HashSet::new();
        mutated_contracts_radius.insert(contract_id);
        StacksTransactionData {
            transaction_identifier: TransactionIdentifier {
                hash: "0".into()
            },
            operations: vec![],
            metadata: StacksTransactionMetadata {
                success,
                result: "".into(),
                receipt: StacksTransactionReceipt {
                    mutated_contracts_radius,
                    mutated_assets_radius: HashSet::new(),
                    events: vec![],
                },
                description: "".into(),
            }
        }
    }

    fn block_with_transactions(transactions: Vec<StacksTransactionData>) -> StacksBlockData {
        StacksBlockData {
            block_identifier: BlockIdentifier { index: 1, hash: "1".into() },
            parent_block_identifier: BlockIdentifier { index: 0, hash: "0".into() },
            timestamp: 0,
            transactions,
            metadata: StacksBlockMetadata { 
                bitcoin_anchor_block_identifier: BlockIdentifier { index: 0, hash: "0".into() }, 
                pox_cycle_index: 0, 
                pox_cycle_position: 0,
                pox_cycle_length: 0 
            }
        }
    }

    #[test]
    fn spawn_integrated_supervisor() {

        use crate::types::{ContractSettings, ContractsObserverConfig, ProjectMetadata, ContractsObserverId};
        use crate::actors::run_supervisor;
        use clarinet_lib::clarity_repl::clarity::types::{StandardPrincipalData, QualifiedContractIdentifier};
        use clarinet_lib::types::StacksChainEvent;
        use std::collections::{BTreeMap};
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
        
        let handle = std::thread::spawn(|| {
            let mut working_dir = std::env::current_dir().unwrap();
            working_dir.push("tests");
            run_supervisor(StorageDriver::filesystem(working_dir), rx)
        });

        let clarion_manifest = ContractsObserverConfig {
            identifier: ContractsObserverId(0),
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

        tx.send(ClarionSupervisorMessage::RegisterContractsObserver(clarion_manifest)).unwrap();

        let block = block_with_transactions(vec![
            transaction_impacting_contract_id(test_contract_id.to_string(), false)
        ]);
        tx.send(ClarionSupervisorMessage::ProcessStacksChainEvent(StacksChainEvent::ChainUpdatedWithBlock(block))).unwrap();

        let delay = time::Duration::from_millis(100);
        let now = time::Instant::now();
        thread::sleep(delay);

        tx.send(ClarionSupervisorMessage::Exit).unwrap();

        let _res = handle.join().unwrap();
    }
}

