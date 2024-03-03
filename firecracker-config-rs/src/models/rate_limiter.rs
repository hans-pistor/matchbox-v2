use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Defines a token bucket with a maximum capacity (size), an initial burst size (`one_time_burst`),
/// and an interval for refilling purposes (`refill_time`). The `refill_rate` is derived from `size` and
/// `refill_time`, and it is the constant rate at which the tokens replenish. The refill process
/// only starts happening after the initial burst budget is consumed.
/// Consumption from the token bucket is unbounded in speed which allows for bursts bound in size
/// by the amount of tokens available. Once the token bucket is empty, consumption speed is bound
/// by the `refill_rate`.
pub struct TokenBucket {
    /// The total number of tokens this bucket can hold
    pub size: u64,
    /// The amount of milliseconds it takes for the bucket to refill
    pub refill_time: u64,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The initial size of a token bucket
    pub one_time_burst: Option<u64>
}

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Defines an IO rate limiter with independent bytes/s and ops/s limits.
/// Limits are defined by configuring each of the _bandwidth_ and _ops_ token buckets.
pub struct RateLimiter {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Token bucket with bytes as tokens
    pub bandwidth: Option<TokenBucket>,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Token bucket with operations as tokens
    pub ops: Option<TokenBucket>,
}
