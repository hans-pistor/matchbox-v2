use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use matchbox::{
    dependency::DependencyFactory,
    sandbox::id::{ProvideIdentifier, VmIdentifier},
};

use crate::common::{ping, wait_until};

mod common;

#[tokio::test]
#[ignore]
async fn test_spawning_a_uvm() {
    let dependency_factory = DependencyFactory::default();
    let factory = dependency_factory.sandbox_provider();

    let sandbox = factory
        .provide_sandbox()
        .await
        .expect("failed to create sandbox");

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
            VmIdentifier::new(format!("vm-{id}"), id)
        }
    }
    let dependency_factory = DependencyFactory::default()
        .with_identifier_provider(Arc::new(Box::new(MockIdentifierFactory {})));

    let factory = dependency_factory.sandbox_provider();

    let sandbox = factory
        .provide_sandbox()
        .await
        .expect("failed to create sandbox");

    let uvm_ip = sandbox.network().microvm_ip();
    println!("Connecting to IP {uvm_ip}");
    wait_until(Duration::from_secs(10), || ping(&uvm_ip)).expect("failed to ping microvm");

    let other = factory
        .provide_sandbox()
        .await
        .expect("failed to create sandbox");

    let other_ip = other.network().microvm_ip();
    println!("Connecting to IP {other_ip}");
    wait_until(Duration::from_secs(10), || ping(&other_ip)).expect("failed to ping other microvm");

    ping(&uvm_ip).expect("The original uVM should still be reachable");
    ping(&other_ip).expect("The next door uVM should still be reachable");
}
