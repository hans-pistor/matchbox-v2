use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::models::rate_limiter::RateLimiter;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum CacheType {
    #[default]
    Unsafe,
    Writeback,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum IoEngine {
    #[default]
    Sync,
    Async,
}

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Represent a block drive
pub struct Drive {
    /// Identifier of the block device
    pub drive_id: String,
    /// Is this the root block device
    pub is_root_device: bool,
    #[builder(default = "CacheType::Unsafe")]
    /// Caching strategy for the block device
    pub cache_type: CacheType,
    #[builder(default = "IoEngine::Sync")]
    /// Type of IO engine used by the block device. "Async" is only supported on host kernels newer
    /// than 5.10.51
    pub io_engine: IoEngine,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Represent the unique id of the boot partition of this device. It is optional and it will be
    /// taken into account only if the `is_root_device` field is true.
    pub partuuid: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Is the block device read only
    pub is_read_only: Option<bool>,
    /// Host level path for the block drive
    pub path_on_host: PathBuf,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rate limiter for operations on the block drive
    pub rate_limiter: Option<RateLimiter>,
}
