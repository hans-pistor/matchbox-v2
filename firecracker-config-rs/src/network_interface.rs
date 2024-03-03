use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::rate_limiter::RateLimiter;

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Defines a network interface
struct NetworkInterface {
    /// Host level path for the guest network interface
    host_dev_name: String,
    /// Identifier for the network interface
    iface_id: String,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// MAC address of the guest
    guest_mac: Option<String>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rate limiter for receiving packets
    rx_rate_limiter: Option<RateLimiter>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rate limiter for sending packets
    tx_rate_limiter: Option<RateLimiter>
}
