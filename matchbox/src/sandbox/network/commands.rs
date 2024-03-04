use std::process::{Command, Output};

use anyhow::Context;

pub enum IpCommand {
    // Corresponds to ip link del $dev
    DeleteDevice { device: String },
    CreateTapDevice { device: String },
    AddAddress { cidr_block: String, device: String },
    Activate { device: String },
}

impl IpCommand {
    pub fn output(self) -> anyhow::Result<Output> {
        let mut cmd = Command::from(self);
        cmd.output().context("Failed to run command")
    }
}

impl From<IpCommand> for Command {
    fn from(value: IpCommand) -> Self {
        let mut cmd = Command::new("ip");
        let _ = match value {
            IpCommand::DeleteDevice { device } => cmd.args(["link", "del", &device]),
            IpCommand::CreateTapDevice { device } => {
                cmd.args(["tuntap", "add", "dev", &device, "mode", "tap"])
            }
            IpCommand::AddAddress { cidr_block, device } => {
                cmd.args(["addr", "add", &cidr_block, "dev", &device])
            }
            IpCommand::Activate { device } => cmd.args(["link", "set", "dev", &device, "up"]),
        };

        cmd
    }
}

#[derive(Debug)]
pub enum Table {
    Forward,
}

impl AsRef<str> for Table {
    fn as_ref(&self) -> &str {
        match self {
            Table::Forward => "FORWARD",
        }
    }
}

#[derive(Debug)]
pub enum Target {
    Accept,
}
impl AsRef<str> for Target {
    fn as_ref(&self) -> &str {
        match self {
            Target::Accept => "ACCEPT",
        }
    }
}

pub enum IpTablesCommand {
    DeleteRule {
        table: Table,
        target: Target,
        input: String,
        output: String,
    },
    AddRule {
        table: Table,
        target: Target,
        input: String,
        output: String,
    },
}

impl IpTablesCommand {
    pub fn output(self) -> anyhow::Result<Output> {
        let mut cmd = Command::from(self);
        cmd.output().context("Failed to run command")
    }
}

impl From<IpTablesCommand> for Command {
    fn from(value: IpTablesCommand) -> Self {
        let mut cmd = Command::new("iptables");
        let _ = match value {
            IpTablesCommand::DeleteRule {
                table,
                target,
                input,
                output,
            } => cmd.args([
                "-D",
                table.as_ref(),
                "-i",
                &input,
                "-o",
                &output,
                "-j",
                target.as_ref(),
            ]),
            IpTablesCommand::AddRule {
                table,
                target,
                input,
                output,
            } => cmd.args([
                "-I",
                table.as_ref(),
                "1",
                "-i",
                &input,
                "-o",
                &output,
                "-j",
                target.as_ref(),
            ]),
        };

        cmd
    }
}
