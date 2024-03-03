use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::rate_limiter::RateLimiter;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum CacheType {
    #[default]
    Unsafe,
    Writeback
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum IoEngine {
    #[default]
    Sync,
    Async
}

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Represent a block drive
pub struct Drive {
    /// Identifier of the block device
    drive_id: String,
    /// Is this the root block device
    is_root_device: bool,
    #[builder(default = "CacheType::Unsafe")]
    /// Caching strategy for the block device
    cache_type: CacheType,
    #[builder(default = "IoEngine::Sync")]
    /// Type of IO engine used by the block device. "Async" is only supported on host kernels newer
    /// than 5.10.51
    io_engine: IoEngine,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Represent the unique id of the boot partition of this device. It is optional and it will be
    /// taken into account only if the `is_root_device` field is true.
    partuuid: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Is the block device read only
    is_read_only: Option<bool>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Host level path for the block drive
    path_on_host: Option<PathBuf>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rate limiter for operations on the block drive
    rate_limiter: Option<RateLimiter>,

}
