pub mod supervisor;
pub mod contract_processor;
pub mod contract_processor_observer;
pub use supervisor::{ClarionSupervisor, ClarionSupervisorMessage};

use std::sync::mpsc::{Receiver};

use kompact::prelude::*;
use std::sync::Arc;

pub fn run_clarion_supervisor(
    supervisor_cmd_rx: Receiver<ClarionSupervisorMessage>,
) -> Result<(), String> {
    match block_on(do_run_clarion_supervisor(
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

pub async fn do_run_clarion_supervisor(
    supervisor_cmd_rx: Receiver<ClarionSupervisorMessage>,
) -> Result<(), String> {
    let system = KompactConfig::default().build().expect("system");

    let supervisor: Arc<Component<ClarionSupervisor>> = system.create(|| ClarionSupervisor::new() );
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

#[test]
fn spawn_integrated_supervisor() {
    use self::supervisor::{ContractSettings, ClarionManifest, ProjectMetadata};
    use clarinet_lib::clarity_repl::clarity::types::{StandardPrincipalData, QualifiedContractIdentifier};
    use std::collections::{BTreeMap};
    use std::convert::TryInto;
    use std::sync::mpsc::channel;
    
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
        run_clarion_supervisor(rx)
    });

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

    tx.send(ClarionSupervisorMessage::RegisterManifest(clarion_manifest)).unwrap();
    tx.send(ClarionSupervisorMessage::Exit).unwrap();

    let _res = handle.join().unwrap();
}
