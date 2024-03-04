use anyhow::Context;
use firecracker_config_rs::models::network_interface::NetworkInterface;
use netns_rs::NetNs;
use std::{path::Path, process::Command};

use self::commands::{IpCommand, IpTablesCommand, Table, Target};

mod commands;

pub const HOST_INTERFACE_NAME: &str = "ens4";
pub const DEFAULT_CIDR_BLOCK: &str = "172.16.0.1/30";

#[derive(Debug)]
pub struct Network {
    id: String,
    netns: NetNs,
}

impl Network {
    pub fn new(id: String, interfaces: &[NetworkInterface]) -> anyhow::Result<Network> {
        let netns = NetNs::new(&id)?;
        let network = Self { id, netns };
        network.setup(interfaces)?;

        Ok(network)
    }

    pub fn netns_path(&self) -> &Path {
        self.netns.path()
    }

    fn setup(&self, interfaces: &[NetworkInterface]) -> anyhow::Result<()> {
        self.netns
            .run(|_| {
                for interface in interfaces {
                    setup_tap_device(&interface.host_dev_name).unwrap();
                }
            })
            .context("failed to run commands inside network namespace")
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        let netns = NetNs::get(&self.id).unwrap();
        netns.remove().unwrap();
    }
}

fn setup_tap_device(device: &str) -> anyhow::Result<()> {
    Command::from(IpCommand::CreateTapDevice {
        device: device.to_string(),
    })
    .output()
    .unwrap();

    Command::from(IpCommand::AddAddress {
        cidr_block: DEFAULT_CIDR_BLOCK.into(),
        device: device.to_string(),
    })
    .output()
    .unwrap();

    Command::from(IpCommand::Activate {
        device: device.to_string(),
    })
    .output()
    .unwrap();

    Command::from(IpTablesCommand::AddRule {
        table: Table::Forward,
        target: Target::Accept,
        input: device.to_string(),
        output: HOST_INTERFACE_NAME.into(),
    })
    .output()
    .unwrap();
    Ok(())
}
