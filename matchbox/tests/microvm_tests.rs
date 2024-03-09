use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use matchbox::{
    dependency::DependencyFactory,
    jailer::factory::JailedFirecrackerFactory,
    sandbox::{
        id::{ProvideIdentifier, VmIdentifier, VmIdentifierFactory},
        spark::factory::SparkClientFactory,
        SandboxFactory, SandboxInitializer,
    },
};

use crate::common::{ping, wait_until};

mod common;

#[tokio::test]
#[ignore]
async fn test_spawning_a_uvm() {
    let dependency_factory = DependencyFactory::default();
    let factory = dependency_factory.sandbox_provider();

    let mut sandbox = factory
        .provide_sandbox()
        .await
        .expect("failed to create sandbox");
    sandbox.start().await.expect("failed to start sandbox");

    let uvm_ip = sandbox.network().microvm_ip();
    println!("Connecting to IP {uvm_ip}");
    wait_until(Duration::from_secs(10), || ping(&uvm_ip)).expect("failed to ping microvm");
}

#[tokio::test]
#[ignore]
async fn test_spawning_next_door_vms() {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    #[derive(Debug)]
    struct MockIdentifierFactory;
    impl ProvideIdentifier for MockIdentifierFactory {
        fn provide_identifier(&self) -> VmIdentifier {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            VmIdentifier::new(id.to_string(), id)
        }
    }
    let dependency_factory = DependencyFactory::default()
        .with_identifier_provider(Arc::new(Box::new(MockIdentifierFactory {})));

    let factory = dependency_factory.sandbox_provider();

    let mut sandbox = factory
        .provide_sandbox()
        .await
        .expect("failed to create sandbox");
    sandbox.start().await.expect("failed to start sandbox");

    let uvm_ip = sandbox.network().microvm_ip();
    println!("Connecting to IP {uvm_ip}");
    wait_until(Duration::from_secs(10), || ping(&uvm_ip)).expect("failed to ping microvm");
}
