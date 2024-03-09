use std::{path::PathBuf};

use self::client::FirecrackerClient;

pub mod client;
pub mod config;
pub mod factory;

#[derive(Debug)]
pub struct FirecrackerProcess {
    pub path_resolver: PathResolver,
    pub client: FirecrackerClient,
}

#[derive(Debug)]
pub struct PathResolver {
    root_directory: PathBuf,
}

impl PathResolver {
    pub fn resolve(&self, root_directory: impl Into<PathBuf>) -> PathBuf {
        let jailed_path = root_directory.into();
        let jailed_path = jailed_path.strip_prefix("/").unwrap();
        self.root_directory.join(jailed_path)
    }
}
