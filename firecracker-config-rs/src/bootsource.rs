use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Boot source descriptor.
struct BootSource {
    /// Host level path to the kernel image used to boot the guest
    kernel_image_path: PathBuf,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Kernel boot arguments
    boot_args: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Host level path to the initrd image used to boot the guest
    initrd_path: Option<PathBuf>,
}
