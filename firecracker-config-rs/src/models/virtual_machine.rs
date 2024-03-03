use super::{
    bootsource::BootSource, drive::Drive, logger::Logger, network_interface::NetworkInterface,
};
use derive_builder::Builder;

#[derive(Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into, strip_option), default)]
pub struct VirtualMachine {
    pub logger: Option<Logger>,
    pub boot_source: BootSource,
    pub drives: Vec<Drive>,
    pub network_interfaces: Vec<NetworkInterface>,
}
