use jailer::factory::JailedFirecrackerFactory;
use sandbox::SandboxFactory;
use server::{Application, ApplicationState};

use crate::sandbox::SandboxInitializer;

pub mod jailer;
pub mod sandbox;
pub mod util;
pub mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let factory = JailedFirecrackerFactory::new(
        "/usr/local/bin/jailer",
        "/usr/local/bin/firecracker",
        "/tmp/vms",
    );
    let sandbox_initializer = SandboxInitializer::new("/tmp/rootfs.ext4", "/tmp/kernel.bin");
    let sandbox_factory = SandboxFactory::new(factory, sandbox_initializer);

    let app = Application::new("0.0.0.0:3000", ApplicationState::new(sandbox_factory)).await?;
    app.run().await?;

    Ok(())
}

