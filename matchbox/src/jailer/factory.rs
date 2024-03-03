use super::{
    client::FirecrackerClient, config::JailerConfigBuilder, JailedFirecracker, JailedPathResolver,
};
use std::{path::PathBuf, process::Command};
use uuid::Uuid;

pub struct JailedFirecrackerFactory {
    jailer_path: PathBuf,
    firecracker_path: PathBuf,
    chroot_base_dir: PathBuf,
}

impl JailedFirecrackerFactory {
    pub fn new(
        jailer_path: impl Into<PathBuf>,
        firecracker_path: impl Into<PathBuf>,
        chroot_base_dir: impl Into<PathBuf>,
    ) -> JailedFirecrackerFactory {
        let jailer_path = jailer_path.into();
        let firecracker_path = firecracker_path.into();
        let chroot_base_dir = chroot_base_dir.into();

        JailedFirecrackerFactory {
            jailer_path,
            firecracker_path,
            chroot_base_dir,
        }
    }

    pub fn spawn_jailed_firecracker(&self, vm_id: Uuid) -> JailedFirecracker {
        let jailer_config = JailerConfigBuilder::default()
            .jailer_path(&self.jailer_path)
            .exec_file(&self.firecracker_path)
            .chroot_base_dir(&self.chroot_base_dir)
            .id(vm_id)
            .build()
            .unwrap();

        let mut cmd = Command::new("tmux");
        cmd.args([
            "new-session",
            "-d",
            "-s",
            &vm_id.to_string(),
            &jailer_config.jailer_path.to_string_lossy(),
            "--id",
            &vm_id.to_string(),
            "--exec-file",
            &jailer_config.exec_file.to_string_lossy(),
            "--gid",
            &u32::from(&jailer_config.gid).to_string(),
            "--uid",
            &u32::from(&jailer_config.uid).to_string(),
            "--chroot-base-dir",
            &jailer_config.chroot_base_dir.to_string_lossy(),
        ]);

        let process = cmd.spawn().unwrap();
        let root_directory = self
            .chroot_base_dir
            .join(&self.firecracker_path.file_stem().unwrap())
            .join(vm_id.to_string())
            .join("root");
        let resolver = JailedPathResolver { root_directory };

        let firecracker_socket = resolver.resolve("/run/firecracker.socket");
        let client = FirecrackerClient::new(firecracker_socket);

        JailedFirecracker {
            process,
            path_resolver: resolver,
            client,
        }
    }
}
