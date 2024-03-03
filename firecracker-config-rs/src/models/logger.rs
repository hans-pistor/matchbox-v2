use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    #[default]
    Info,
    Warning,
    Error
}

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Describes the configuration options for the logging capability
pub struct Logger {
    #[builder(setter(strip_option), default = "LogLevel::Info")]
    /// Set the level
    level: LogLevel,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Path to the named pipe or file for human readable log output
    log_path: Option<PathBuf>,
    #[builder(setter(strip_option), default = "false")]
    /// Output the level in the logs
    show_level: bool,
    #[builder(setter(strip_option), default = "false")]
    /// Include the file path and line number of the log's origin
    show_log_origin: bool,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The moduel path to filter log messagse by
    module: Option<String>
}
