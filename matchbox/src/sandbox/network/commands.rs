use std::process::{Command, Output};

use anyhow::Context;

pub enum IpCommand {
    // Corresponds to ip link del $dev
    DeleteDevice { device: String },
    CreateTapDevice { device: String },
    AddAddress { cidr_block: String, device: String },
    Activate { device: String },
    CreateVethPair { veth: String, vpeer: String },
    MoveDeviceToNamespace { device: String, namespace: String },
    AddDefaultRoute { address: String },
    AddRoute { to: String, via: String },
}

impl IpCommand {
    pub fn output(self) -> anyhow::Result<Output> {
        let mut cmd = Command::from(self);
        let output = cmd.output().context("Failed to run command")?;

        if !output.status.success() {
            println!(
                "Command {} {} failed: stdout = {}. stderr = {}.",
                cmd.get_program().to_string_lossy(),
                cmd.get_args()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }

        Ok(output)
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
            IpCommand::CreateVethPair {
                veth: device_one,
                vpeer: device_two,
            } => cmd.args([
                "link",
                "add",
                &device_two,
                "type",
                "veth",
                "peer",
                "name",
                &device_one,
            ]),
            IpCommand::MoveDeviceToNamespace { device, namespace } => {
                cmd.args(["link", "set", &device, "netns", &namespace])
            }
            IpCommand::AddDefaultRoute { address } => {
                cmd.args(["route", "add", "default", "via", &address])
            }
            IpCommand::AddRoute { to, via } => cmd.args(["route", "add", &to, "via", &via]),
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
    EnableMasquerade {
        source_address: Option<String>,
        output: String,
    },
    DisableMasquerade {
        source_address: Option<String>,
        output: String,
    },
    RewriteSource {
        output: String,
        source: String,
        to: String,
    },
    RewriteDestination {
        input: String,
        destination: String,
        to: String,
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
                "-A",
                table.as_ref(),
                "-i",
                &input,
                "-o",
                &output,
                "-j",
                target.as_ref(),
            ]),
            IpTablesCommand::EnableMasquerade {
                source_address,
                output,
            } => {
                cmd.args(["-t", "nat", "-A", "POSTROUTING"]);
                if let Some(source) = source_address {
                    cmd.args(["--source", &source]);
                }
                cmd.args(["-o", &output, "-j", "MASQUERADE"])
            }
            IpTablesCommand::DisableMasquerade {
                source_address,
                output,
            } => {
                cmd.args(["-t", "nat", "-D", "POSTROUTING"]);
                if let Some(source) = source_address {
                    cmd.args(["--source", &source]);
                }
                cmd.args(["-o", &output, "-j", "MASQUERADE"])
            }
            IpTablesCommand::RewriteSource { output, source, to } => cmd.args([
                "-t",
                "-nat",
                "-A",
                "POSTROUTING",
                "-o",
                &output,
                "-s",
                &source,
                "-j",
                "SNAT",
                "--to",
                &to,
            ]),
            IpTablesCommand::RewriteDestination {
                input,
                destination,
                to,
            } => cmd.args([
                "-t",
                "nat",
                "-A",
                "PREROUTING",
                "-i",
                &input,
                "-d",
                &destination,
                "-j",
                "DNAT",
                "--to",
                &to,
            ]),
        };

        cmd
    }
}
