use derive_builder::Builder;
use super::{bootsource::BootSource, drive::Drive, logger::Logger, network_interface::NetworkInterface};

#[derive(Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into, strip_option), default)]
struct VirtualMachine {
    logger: Option<Logger>,
    boot_source: BootSource,
    drives: Vec<Drive>,
    network_interfaces: Vec<NetworkInterface>
}
