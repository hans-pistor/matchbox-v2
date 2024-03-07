use std::time::Duration;

use matchbox::{
    jailer::factory::JailedFirecrackerFactory,
    sandbox::{SandboxFactory, SandboxInitializer},
};

use crate::common::{ping, wait_until};

mod common;

#[tokio::test]
#[ignore]
async fn test_spawning_a_uvm() {
    let factory = JailedFirecrackerFactory::new(
        "/usr/local/bin/jailer",
        "/usr/local/bin/firecracker",
        "/tmp/vms",
    );
    let sandbox_initializer = SandboxInitializer::new("/tmp/rootfs.ext4", "/tmp/kernel.bin");
    let factory = SandboxFactory::new(factory, sandbox_initializer);

    let mut sandbox = factory
        .spawn_sandbox()
        .await
        .expect("failed to create sandbox");
    sandbox.start().await.expect("failed to start sandbox");

    let uvm_ip = sandbox.network().microvm_ip();
    println!("Connecting to IP {uvm_ip}");
    wait_until(Duration::from_secs(10), || ping(&uvm_ip)).expect("failed to ping microvm");
}
