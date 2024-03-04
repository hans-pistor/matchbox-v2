use anyhow::Context;
use firecracker_config_rs::models::network_interface::NetworkInterface;
use netns_rs::NetNs;
use std::path::PathBuf;

use self::commands::{IpCommand, IpTablesCommand, Table, Target};

use super::id::VmIdentifier;

mod commands;

pub const HOST_INTERFACE_NAME: &str = "ens4";
pub const DEFAULT_CIDR_BLOCK: &str = "172.16.0.1/30";

#[derive(Debug)]
pub struct Network {
    namespace_name: String,
    address_start: u64,
}

impl Network {
    pub fn new(id: &VmIdentifier, interfaces: &[NetworkInterface]) -> anyhow::Result<Network> {
        // Create the network namespace
        let _ = NetNs::new(id.id())?;
        let network = Self {
            namespace_name: id.id().into(),
            address_start: id.counter(),
        };
        network.setup(interfaces)?;

        Ok(network)
    }

    pub fn netns_path(&self) -> anyhow::Result<PathBuf> {
        NetNs::get(&self.namespace_name)
            .map(|ns| ns.path().to_owned())
            .context("Failed to get network namespace")
    }

    fn setup(&self, interfaces: &[NetworkInterface]) -> anyhow::Result<()> {
        let netns = NetNs::get(&self.namespace_name)?;
        self.setup_interfaces(&netns, interfaces)?;
        self.setup_veth_devices(&netns)?;

        Ok(())
    }

    fn veth(&self) -> (String, String) {
        let veth_name = format!("{}-veth", self.namespace_name);
        let veth_address = format!("10.200.{}.10", self.address_start);
        (veth_name, veth_address)
    }

    fn vpeer(&self) -> (String, String) {
        let vpeer_name = format!("{}-vpeer", self.namespace_name);
        let vpeer_address = format!("10.200.{}.11", self.address_start);
        (vpeer_name, vpeer_address)
    }

    fn setup_interfaces(
        &self,
        netns: &NetNs,
        interfaces: &[NetworkInterface],
    ) -> anyhow::Result<()> {
        netns.run(|_| {
            for interface in interfaces {
                setup_tap_device(&interface.host_dev_name)?;
            }

            Ok::<(), anyhow::Error>(())
        })??;

        Ok(())
    }

    fn setup_veth_devices(&self, netns: &NetNs) -> anyhow::Result<()> {
        let (veth_device_name, host_address) = self.veth();
        let (vpeer_device_name, peer_address) = self.vpeer();

        // Create the veth pair
        IpCommand::CreateVethPair {
            veth: veth_device_name.clone(),
            vpeer: vpeer_device_name.clone(),
        }
        .output()?;

        // Move vpeer into the guest network namespace
        IpCommand::MoveDeviceToNamespace {
            device: vpeer_device_name.clone(),
            namespace: self.namespace_name.clone(),
        }
        .output()?;

        // Assign veth an IP address & activate it
        IpCommand::AddAddress {
            cidr_block: format!("{host_address}/24"),
            device: veth_device_name.clone(),
        }
        .output()?;

        IpCommand::Activate {
            device: veth_device_name.clone(),
        }
        .output()?;

        netns.run(|_| {
            // Assign vpeer an ip address and activate it
            IpCommand::AddAddress {
                cidr_block: format!("{peer_address}/24"),
                device: vpeer_device_name.clone(),
            }
            .output()?;

            IpCommand::Activate {
                device: vpeer_device_name.clone(),
            }
            .output()?;

            // Set the default route as veth (which will go through vpeer)
            IpCommand::AddDefaultRoute {
                address: host_address.clone(),
            }
            .output()?;

            // Enable masquerading for all traffic leaving via vpeer
            IpTablesCommand::EnableMasquerade {
                source_address: None,
                output: vpeer_device_name.clone(),
            }
            .output()?;

            Ok::<(), anyhow::Error>(())
        })??;

        IpTablesCommand::AddRule {
            table: Table::Forward,
            target: Target::Accept,
            input: veth_device_name.clone(),
            output: HOST_INTERFACE_NAME.into(),
        }
        .output()?;
        IpTablesCommand::AddRule {
            table: Table::Forward,
            target: Target::Accept,
            input: HOST_INTERFACE_NAME.into(),
            output: veth_device_name.clone(),
        }
        .output()?;

        // Enable masquerading for all traffic coming from the peer address leaving
        // via the host interface (ens4 in my case)
        IpTablesCommand::EnableMasquerade {
            source_address: Some(format!("{peer_address}/24")),
            output: HOST_INTERFACE_NAME.into(),
        }
        .output()?;

        Ok(())
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        let netns = NetNs::get(&self.namespace_name).unwrap();
        netns.remove().unwrap();
        let (veth_device_name, _) = self.veth();

        IpCommand::DeleteDevice {
            device: veth_device_name,
        }
        .output()
        .unwrap();
    }
}

fn setup_tap_device(device: &str) -> anyhow::Result<()> {
    IpCommand::CreateTapDevice {
        device: device.to_string(),
    }
    .output()?;

    IpCommand::AddAddress {
        cidr_block: DEFAULT_CIDR_BLOCK.into(),
        device: device.to_string(),
    }
    .output()?;

    IpCommand::Activate {
        device: device.to_string(),
    }
    .output()?;

    IpTablesCommand::AddRule {
        table: Table::Forward,
        target: Target::Accept,
        input: device.to_string(),
        output: HOST_INTERFACE_NAME.into(),
    }
    .output()?;
    Ok(())
}
