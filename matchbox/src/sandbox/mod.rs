use std::fs::OpenOptions;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use firecracker_config_rs::models::bootsource::BootSourceBuilder;
use firecracker_config_rs::models::drive::DriveBuilder;
use firecracker_config_rs::models::logger::{LogLevel, LoggerBuilder};
use firecracker_config_rs::models::virtual_machine::{VirtualMachine, VirtualMachineBuilder};
use uuid::Uuid;

use crate::jailer::client::Action;
use crate::jailer::factory::JailedFirecrackerFactory;
use crate::jailer::{JailedFirecracker, JailedPathResolver};
use crate::util;

#[derive(Debug, Clone, Copy)]
pub enum SandboxState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug)]
pub struct Sandbox {
    uuid: Uuid,
    state: SandboxState,
    pub jailed_firecracker: JailedFirecracker,
    virtual_machine_config: VirtualMachine,
}

impl Sandbox {
    pub fn id(&self) -> Uuid {
        self.uuid
    }

    pub fn path_resolver(&self) -> &JailedPathResolver {
        &self.jailed_firecracker.path_resolver
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

#[derive(Clone, Debug)]
pub struct SandboxFactory {
    firecracker_factory: JailedFirecrackerFactory,
    sandbox_initializer: SandboxInitializer,
}

impl SandboxFactory {
    pub fn new(
        firecracker_factory: JailedFirecrackerFactory,
        sandbox_initializer: SandboxInitializer,
    ) -> SandboxFactory {
        SandboxFactory {
            firecracker_factory,
            sandbox_initializer,
        }
    }

    pub async fn spawn_sandbox(&self) -> anyhow::Result<Sandbox> {
        let uuid = Uuid::new_v4();
        let jailed_firecracker = self.firecracker_factory.spawn_jailed_firecracker(uuid);
        let virtual_machine_config = VirtualMachineBuilder::default()
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
            .build()?;

        let sandbox = Sandbox {
            uuid,
            state: SandboxState::Stopped,
            jailed_firecracker,
            virtual_machine_config,
        };

        let sandbox = self.sandbox_initializer.initialize(sandbox).await?;

        Ok(sandbox)
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

    pub async fn initialize(&self, sandbox: Sandbox) -> anyhow::Result<Sandbox> {
        self.wait_for_health_check(&sandbox).await?;
        self.setup_logging(&sandbox).await?;
        self.setup_bootsource(&sandbox).await?;
        self.setup_drives(&sandbox).await?;
        self.setup_network_interfaces(&sandbox).await?;

        Ok(sandbox)
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

        // Create the parent direct of the log fileo
        std::fs::create_dir_all(log_file_path_on_host.parent().unwrap())?;

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
        util::copy(&self.kernel_image, kernel_image_path_on_host);

        sandbox
            .jailed_firecracker
            .client
            .put("/boot-source", &bootsource)
            .await?;

        Ok(())
    }

    async fn setup_drives(&self, sandbox: &Sandbox) -> anyhow::Result<()> {
        let rootfs_path = sandbox.path_resolver().resolve("/drives/rootfs.ext4");
        std::fs::create_dir_all(rootfs_path.parent().unwrap())?;
        util::copy(&self.rootfs, rootfs_path);

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
}
