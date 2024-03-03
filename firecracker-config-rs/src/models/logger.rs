use std::path::PathBuf;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum LogLevel {
    Off,
    Trace,
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Builder, Clone, PartialEq, Debug, Default)]
#[builder(setter(into))]
/// Describes the configuration options for the logging capability
pub struct Logger {
    /// Path to the named pipe or file for human readable log output
    pub log_path: PathBuf,
    #[builder(setter(strip_option), default = "LogLevel::Info")]
    /// Set the level
    pub level: LogLevel,
    #[builder(setter(strip_option), default = "false")]
    /// Output the level in the logs
    pub show_level: bool,
    #[builder(setter(strip_option), default = "false")]
    /// Include the file path and line number of the log's origin
    pub show_log_origin: bool,
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The moduel path to filter log messagse by
    pub module: Option<String>,
}
