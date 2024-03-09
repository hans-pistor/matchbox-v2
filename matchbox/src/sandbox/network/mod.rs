use anyhow::Context;
use firecracker_config_rs::models::network_interface::NetworkInterface;
use netns_rs::NetNs;
use std::path::PathBuf;

use self::commands::{IpCommand, IpTablesCommand, Table, Target};

use super::id::{AddressBlock, VmIdentifier};

mod commands;

pub const HOST_INTERFACE_NAME: &str = "ens4";
pub const DEFAULT_CIDR_BLOCK: &str = "172.16.0.1/30";

enum IpAddressType {
    Veth,
    Vpeer,
    Microvm,
}

impl From<IpAddressType> for u64 {
    fn from(value: IpAddressType) -> Self {
        match value {
            IpAddressType::Veth => 0,
            IpAddressType::Vpeer => 1,
            IpAddressType::Microvm => 2,
        }
    }
}

#[derive(Debug)]
pub struct Network {
    namespace_name: String,
    address_block: AddressBlock,
}

impl Network {
    pub fn new(id: &VmIdentifier, interfaces: &[NetworkInterface]) -> anyhow::Result<Network> {
        // Create the network namespace
        let _ = NetNs::new(id.id())?;

        let network = Self {
            namespace_name: id.id().into(),
            address_block: id.address_block().clone(),
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
        self.setup_veth_devices(&netns)?;
        self.setup_interfaces(&netns, interfaces)?;

        Ok(())
    }

    pub fn veth(&self) -> (String, String) {
        let veth_name = format!("{}-veth", self.namespace_name);
        let veth_address = self.address_block.get_ip(IpAddressType::Veth);

        (veth_name, veth_address)
    }

    pub fn vpeer(&self) -> (String, String) {
        let vpeer_name = format!("{}-vpeer", self.namespace_name);
        let vpeer_address = self.address_block.get_ip(IpAddressType::Vpeer);
        (vpeer_name, vpeer_address)
    }

    pub fn microvm_ip(&self) -> String {
        self.address_block.get_ip(IpAddressType::Microvm)
    }

    fn setup_interfaces(
        &self,
        netns: &NetNs,
        interfaces: &[NetworkInterface],
    ) -> anyhow::Result<()> {
        let (veth_device, _) = self.vpeer();
        netns.run(|_| {
            for interface in interfaces {
                setup_tap_device(&interface.host_dev_name, veth_device.clone())?;
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

        // Assign veth an IP address & activate it
        IpCommand::AddAddress {
            cidr_block: format!("{host_address}/29"),
            device: veth_device_name.clone(),
        }
        .output()?;

        IpCommand::Activate {
            device: veth_device_name.clone(),
        }
        .output()?;

        // Move vpeer into the guest network namespace
        IpCommand::MoveDeviceToNamespace {
            device: vpeer_device_name.clone(),
            namespace: self.namespace_name.clone(),
        }
        .output()?;

        netns.run(|_| {
            // Assign vpeer an ip address and activate it
            IpCommand::AddAddress {
                cidr_block: format!("{peer_address}/29"),
                device: vpeer_device_name.clone(),
            }
            .output()?;

            IpCommand::Activate {
                device: vpeer_device_name.clone(),
            }
            .output()?;

            IpCommand::Activate {
                device: "lo".into(),
            }
            .output()?;

            // Set the default route as veth (which will go through vpeer)
            IpCommand::AddDefaultRoute {
                address: host_address.clone(),
            }
            .output()?;

            IpTablesCommand::EnableMasquerade {
                source_address: None,
                output: vpeer_device_name.clone(),
            }
            .output()?;

            IpTablesCommand::RewriteSource {
                output: vpeer_device_name.clone(),
                source: "172.16.0.2".into(),
                to: self.microvm_ip(),
            }
            .output()?;

            IpTablesCommand::RewriteDestination {
                input: vpeer_device_name.clone(),
                destination: self.microvm_ip(),
                to: "172.16.0.2".into(),
            }
            .output()?;

            Ok::<(), anyhow::Error>(())
        })??;

        IpCommand::AddRoute {
            to: self.microvm_ip(),
            via: peer_address.clone(),
        }
        .output()?;

        IpTablesCommand::EnableMasquerade {
            source_address: Some(format!("{}/29", peer_address.clone())),
            output: HOST_INTERFACE_NAME.into(),
        }
        .output()?;

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

        let (veth_name, _) = self.veth();
        let (_, peer_address) = self.vpeer();

        IpTablesCommand::DeleteRule {
            table: Table::Forward,
            target: Target::Accept,
            input: veth_name.clone(),
            output: HOST_INTERFACE_NAME.into(),
        }
        .output()
        .unwrap();

        IpTablesCommand::DeleteRule {
            table: Table::Forward,
            target: Target::Accept,
            output: veth_name,
            input: HOST_INTERFACE_NAME.into(),
        }
        .output()
        .unwrap();

        IpTablesCommand::DisableMasquerade {
            source_address: Some(format!("{peer_address}/29")),
            output: HOST_INTERFACE_NAME.into(),
        }
        .output()
        .unwrap();
    }
}

fn setup_tap_device(device: &str, veth_device: String) -> anyhow::Result<()> {
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
        output: veth_device,
    }
    .output()?;
    Ok(())
}
