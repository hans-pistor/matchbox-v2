use matchbox::jailer::factory::JailedFirecrackerFactory;
use matchbox::sandbox::id::VmIdentifierFactory;
use matchbox::sandbox::spark::factory::SparkClientFactory;
use matchbox::sandbox::SandboxFactory;
use matchbox::server::{Application, ApplicationState};

use matchbox::sandbox::SandboxInitializer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let factory = JailedFirecrackerFactory::new(
        "/usr/local/bin/jailer",
        "/usr/local/bin/firecracker",
        "/tmp/vms",
    );
    let sandbox_initializer = SandboxInitializer::new("/tmp/rootfs.ext4", "/tmp/kernel.bin");
    let sandbox_factory = Box::new(SandboxFactory::new(
        Box::new(VmIdentifierFactory),
        Box::new(SparkClientFactory),
        factory,
        sandbox_initializer,
    ));

    let app = Application::new("0.0.0.0:3000", ApplicationState::new(sandbox_factory)).await?;
    app.run().await?;

    Ok(())
}
