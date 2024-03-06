use std::{process::Command, time::Duration};

use matchbox::sandbox::{id::VmIdentifier, network::Network};
use netns_rs::NetNs;

#[test]
#[ignore]
/// This test will fail on ci probably because its working with networking.
fn test_namespaced_network_communication_works() -> anyhow::Result<()> {
    let id = VmIdentifier::default();
    // Creates the network namespace & relevant configuration
    let _network = Network::new(&id, &[])?;

    let netns = NetNs::get(id.id())?;
    let output = netns.run(|_| {
        println!("Attempting to ping from netns {}", id.id());
        let mut cmd = Command::new("ping");
        cmd.args(["-c", "1", "8.8.8.8"]).output().unwrap()
    })?;

    assert!(
        output.status.success(),
        "We should be able to ping 8.8.8.8 from inside the netns"
    );

    Ok(())
}
