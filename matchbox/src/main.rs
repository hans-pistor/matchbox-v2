use jailer::factory::JailedFirecrackerFactory;
use sandbox::SandboxFactory;

use crate::sandbox::SandboxInitializer;

pub mod jailer;
pub mod sandbox;
pub mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let factory = JailedFirecrackerFactory::new(
        "/usr/local/bin/jailer",
        "/usr/local/bin/firecracker",
        "/tmp/vms",
    );
    let sandbox_initializer = SandboxInitializer::new("/tmp/rootfs.ext4", "/tmp/kernel.bin");
    let factory = SandboxFactory::new(factory, sandbox_initializer);
    let mut sandbox = factory.spawn_sandbox().await?;

    println!("{sandbox:?}");
    sandbox.start().await?;

    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    Ok(())
}
