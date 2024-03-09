use matchbox::sandbox::{id::VmIdentifier, network::Network};
use netns_rs::NetNs;

use crate::common::ping;

mod common;

#[test]
#[ignore]
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

#[test]
#[ignore]
fn test_next_door_namespaces_can_connect_to_internet() -> anyhow::Result<()> {
    let id = VmIdentifier::new("network-1".into(), 1);
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

    let other = VmIdentifier::new("network-2".into(), 2);
    let _other_network = Network::new(&other, &[])?;

    let other_ns = NetNs::get(other.id())?;
    let output = other_ns.run(|_| {
        println!("Attempting to ping from netns {}", other.id());
        ping("8.8.8.8")
    })?;

    assert!(
        output.is_ok(),
        "We should be able to ping 8.8.8.8 from inside the netns"
    );

    Ok(())
}
