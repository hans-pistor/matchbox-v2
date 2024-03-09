use std::fmt::Debug;
use std::fs::OpenOptions;

use std::path::{Path, PathBuf};
use std::process::Command;

use std::sync::Arc;
use std::time::{Duration, Instant};

use derive_builder::Builder;
use firecracker_config_rs::models::bootsource::BootSourceBuilder;
use firecracker_config_rs::models::drive::DriveBuilder;
use firecracker_config_rs::models::logger::{LogLevel, LoggerBuilder};
use firecracker_config_rs::models::network_interface::NetworkInterfaceBuilder;
use firecracker_config_rs::models::virtual_machine::{VirtualMachine, VirtualMachineBuilder};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};

use crate::jailer::client::Action;
use crate::jailer::factory::ProvideFirecracker;
use crate::jailer::{FirecrackerProcess, PathResolver};
use crate::util::{self, copy};

use self::id::{ProvideIdentifier, VmIdentifier};
use self::network::Network;
use self::spark::factory::ProvideSparkClient;
use self::spark::SparkClient;

pub mod id;
pub mod network;
pub mod spark;

#[derive(Debug, Clone, Copy)]
pub enum SandboxState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug)]
pub struct Sandbox {
    id: VmIdentifier,
    state: SandboxState,
    network: Network,
    pub jailed_firecracker: FirecrackerProcess,
    virtual_machine_config: VirtualMachine,
    client: Mutex<SparkClient>,
}

impl Sandbox {
    pub fn id(&self) -> &str {
        self.id.id()
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    pub fn path_resolver(&self) -> &PathResolver {
        &self.jailed_firecracker.path_resolver
    }

    pub async fn client(&self) -> MutexGuard<SparkClient> {
        self.client.lock().await
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.jailed_firecracker
            .client
            .action(Action::InstanceStart)
            .await?;
        self.state = SandboxState::Running;
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let mut cmd = Command::new("tmux");
        cmd.args(["kill-session", "-t", self.id()])
            .output()
            .unwrap();
        let root_directory = self.path_resolver().resolve("/");
        let vm_directory = root_directory.parent().unwrap();
        std::fs::remove_dir_all(vm_directory).unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Location {
    Local { path: String },
    CloudStorage { path: String },
}

impl Location {
    pub fn to_local_path(&self) -> anyhow::Result<PathBuf> {
        match self {
            Location::Local { path } => {
                let path = PathBuf::from(path);
                if !path.exists() {
                    anyhow::bail!("path {} doesn't exist", path.display());
                }

                Ok(path)
            }
            Location::CloudStorage { path } => {
                unimplemented!("Google cloud storage not yet implemented")
            }
        }
    }
}

#[derive(Builder, Debug, Default)]
#[builder(setter(into))]
pub struct ProvideSandboxOptions {
    #[builder(setter(strip_option), default)]
    code_drive_location: Option<Location>,
}

#[async_trait::async_trait]
pub trait ProvideSandbox {
    async fn provide_sandbox(&self, options: ProvideSandboxOptions) -> anyhow::Result<Sandbox>;
}

#[async_trait::async_trait]
impl ProvideSandbox for SandboxFactory {
    async fn provide_sandbox(&self, options: ProvideSandboxOptions) -> anyhow::Result<Sandbox> {
        self.spawn_sandbox(options).await
    }
}

#[derive(Debug)]
pub struct SandboxFactory {
    identifier_factory: Arc<Box<dyn ProvideIdentifier>>,
    spark_factory: Arc<Box<dyn ProvideSparkClient>>,
    firecracker_factory: Arc<Box<dyn ProvideFirecracker>>,
    sandbox_initializer: Arc<Box<dyn InitializeSandbox>>,
    dummy_drive_path: PathBuf,
}

impl SandboxFactory {
    pub fn new(
        identifier_factory: Arc<Box<dyn ProvideIdentifier>>,
        spark_factory: Arc<Box<dyn ProvideSparkClient>>,
        firecracker_factory: Arc<Box<dyn ProvideFirecracker>>,
        sandbox_initializer: Arc<Box<dyn InitializeSandbox>>,
        dummy_drive_path: PathBuf,
    ) -> SandboxFactory {
        SandboxFactory {
            identifier_factory,
            spark_factory,
            firecracker_factory,
            sandbox_initializer,
            dummy_drive_path,
        }
    }

    pub async fn spawn_sandbox(&self, options: ProvideSandboxOptions) -> anyhow::Result<Sandbox> {
        let id = self.identifier_factory.provide_identifier();
        let mut virtual_machine_config = VirtualMachineBuilder::default()
            .logger(
                LoggerBuilder::default()
                    .log_path("/log/firecracker.log")
                    .level(LogLevel::Info)
                    .show_level(true)
                    .show_log_origin(true)
                    .build()?
            )
            .boot_source(
                BootSourceBuilder::default()
                    .kernel_image_path("/kernel.bin")
                    .boot_args("console=ttyS0 reboot=k panic=1 pci=off random.trust_cpu=on IP_ADDRESS::172.16.0.2 IFACE::eth0 GATEWAY::172.16.0.1")
                    .build()?
            )
            .drives(vec![
                DriveBuilder::default()
                    .drive_id("rootfs")
                    .path_on_host("/drives/rootfs.ext4")
                    .is_root_device(true)
                    .is_read_only(false)
                    .build()?
            ])
            .network_interfaces(vec![
                NetworkInterfaceBuilder::default()
                .host_dev_name("tap0")
                .iface_id("eth0")
                .guest_mac("06:00:AC:10:00:02")
                .build()?
            ])
            .build()?;
        let network = Network::new(&id, &virtual_machine_config.network_interfaces)?;
        let jailed_firecracker = self
            .firecracker_factory
            .provide_firecracker(id.id(), &network.netns_path()?);

        let mut sandbox = Sandbox {
            id,
            state: SandboxState::Stopped,
            jailed_firecracker,
            virtual_machine_config,
            client: Mutex::new(
                self.spark_factory
                    .provide_spark_client(&network.microvm_ip())
                    .await?,
            ),
            network,
        };

        copy_if_exists(
            &options.code_drive_location,
            sandbox.path_resolver().resolve("/drives/code-drive.ext4"),
            &self.dummy_drive_path,
        )?;
        sandbox.virtual_machine_config.drives.push(
            DriveBuilder::default()
                .drive_id("vdb")
                .path_on_host("/drives/code-drive.ext4")
                .is_root_device(false)
                .is_read_only(false)
                .build()?,
        );

        self.sandbox_initializer
            .initialize_sandbox(&mut sandbox)
            .await?;

        Ok(sandbox)
    }
}

fn copy_if_exists(file: &Option<Location>, path: PathBuf, dummy_path: &Path) -> anyhow::Result<()> {
    match file {
        Some(file) => copy(file.to_local_path()?, path),
        None => copy(dummy_path, path),
    }
}

#[async_trait::async_trait]
pub trait InitializeSandbox: Debug + Send + Sync {
    async fn initialize_sandbox(&self, sandbox: &mut Sandbox) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl InitializeSandbox for SandboxInitializer {
    async fn initialize_sandbox(&self, sandbox: &mut Sandbox) -> anyhow::Result<()> {
        self.initialize(sandbox).await
    }
}

#[derive(Clone, Debug)]
pub struct SandboxInitializer {
    rootfs: PathBuf,
    kernel_image: PathBuf,
}

impl SandboxInitializer {
    pub fn new(rootfs: impl Into<PathBuf>, kernel_image: impl Into<PathBuf>) -> SandboxInitializer {
        Self {
            rootfs: rootfs.into(),
            kernel_image: kernel_image.into(),
        }
    }

    pub async fn initialize(&self, sandbox: &mut Sandbox) -> anyhow::Result<()> {
        self.wait_for_health_check(sandbox).await?;
        self.setup_logging(sandbox).await?;
        self.setup_bootsource(sandbox).await?;
        self.setup_drives(sandbox).await?;
        self.setup_network_interfaces(sandbox).await?;

        sandbox.start().await?;

        self.wait_for_spark_health_check(sandbox).await?;
        self.mount_drives_in_guest(sandbox).await?;
        Ok(())
    }

    async fn wait_for_health_check(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        let health_check = || async {
            let response = sandbox.jailed_firecracker.client.get("/version").await?;

            match response.status().is_success() {
                true => Ok(()),
                false => anyhow::bail!("Received error from health check API"),
            }
        };

        let start = Instant::now();
        while health_check().await.is_err() {
            if start.elapsed() > Duration::from_secs(20) {
                panic!("Firecracker did not become healthy in 20 seconds");
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }

    async fn setup_logging(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        let logger = match &sandbox.virtual_machine_config.logger {
            Some(config) => config.clone(),
            None => return Ok(()),
        };

        let log_file_path_on_host = sandbox.path_resolver().resolve(&logger.log_path);

        // Touch the logfile
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path_on_host)?;

        sandbox
            .jailed_firecracker
            .client
            .put("/logger", &logger)
            .await?;

        Ok(())
    }

    async fn setup_bootsource(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        let bootsource = &sandbox.virtual_machine_config.boot_source;

        let kernel_image_path_on_host = sandbox
            .path_resolver()
            .resolve(&bootsource.kernel_image_path);
        std::fs::create_dir_all(kernel_image_path_on_host.parent().unwrap())?;

        // Copy the global bootsource into the VM directory
        util::copy(&self.kernel_image, kernel_image_path_on_host)?;

        sandbox
            .jailed_firecracker
            .client
            .put("/boot-source", &bootsource)
            .await?;

        Ok(())
    }

    async fn setup_drives(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        let rootfs_path = sandbox.path_resolver().resolve("/drives/rootfs.ext4");
        util::copy(&self.rootfs, rootfs_path)?;

        for drive in &sandbox.virtual_machine_config.drives {
            let path = format!("/drives/{}", drive.drive_id);

            sandbox.jailed_firecracker.client.put(path, &drive).await?;
        }

        Ok(())
    }

    async fn setup_network_interfaces(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        for interface in &sandbox.virtual_machine_config.network_interfaces {
            let path = format!("/network-interfaces/{}", interface.iface_id);
            sandbox
                .jailed_firecracker
                .client
                .put(path, &interface)
                .await?;
        }

        Ok(())
    }

    async fn wait_for_spark_health_check(&self, sandbox: &mut Sandbox) -> anyhow::Result<()> {
        let mut client = sandbox.client().await;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            if client.health_check().await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        anyhow::bail!("sandbox {} spark-server never became healthy", sandbox.id())
    }

    async fn mount_drives_in_guest(&self, sandbox: &mut Sandbox) -> anyhow::Result<()> {
        let mut client = sandbox.client().await;
        for drive in &sandbox.virtual_machine_config.drives {
            if drive.drive_id == "rootfs" {
                continue;
            }

            client
                .mount_drive(
                    format!("/dev/{}", drive.drive_id),
                    format!("/tmp/{}", drive.drive_id),
                )
                .await?;
        }
        Ok(())
    }
}
