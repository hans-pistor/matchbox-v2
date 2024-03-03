use std::{path::PathBuf, process::Child};

use self::client::FirecrackerClient;

pub mod client;
pub mod config;
pub mod factory;

#[derive(Debug)]
pub struct JailedFirecracker {
    process: Child,
    pub path_resolver: JailedPathResolver,
    pub client: FirecrackerClient,
}

#[derive(Debug)]
pub struct JailedPathResolver {
    root_directory: PathBuf,
}

impl JailedPathResolver {
    pub fn resolve(&self, jailed_path: impl Into<PathBuf>) -> PathBuf {
        let jailed_path = jailed_path.into();
        let jailed_path = jailed_path.strip_prefix("/").unwrap();
        self.root_directory.join(jailed_path)
    }
}