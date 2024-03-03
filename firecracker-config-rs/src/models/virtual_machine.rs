use derive_builder::Builder;
use super::{bootsource::BootSource, drive::Drive, logger::Logger, network_interface::NetworkInterface};

#[derive(Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into, strip_option), default)]
struct VirtualMachine {
    pub logger: Option<Logger>,
    pub boot_source: BootSource,
    pub drives: Vec<Drive>,
    pub network_interfaces: Vec<NetworkInterface>
}
