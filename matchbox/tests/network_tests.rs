use matchbox::sandbox::{id::VmIdentifier, network::Network};
use netns_rs::NetNs;

use crate::common::ping;

mod common;

#[test]
/// This test will fail on ci probably because its working with networking.
fn test_namespaced_network_communication_works() -> anyhow::Result<()> {
    let id = VmIdentifier::default();
    println!("Creating network with id {id:?}");
    // Creates the network namespace & relevant configuration
    let _network = Network::new(&id, &[])?;

    let netns = NetNs::get(id.id())?;
    let output = netns.run(|_| {
        println!("Attempting to ping from netns {}", id.id());
        ping("8.8.8.8")
    })?;

    assert!(
        output.is_ok(),
        "We should be able to ping 8.8.8.8 from inside the netns"
    );

    Ok(())
}
