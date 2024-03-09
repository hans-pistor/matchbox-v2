use matchbox::dependency::DependencyFactory;
use matchbox::jailer::factory::JailedFirecrackerFactory;
use matchbox::sandbox::id::VmIdentifierFactory;
use matchbox::sandbox::spark::factory::SparkClientFactory;
use matchbox::sandbox::SandboxFactory;
use matchbox::server::{Application, ApplicationState};

use matchbox::sandbox::SandboxInitializer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dependency_factory = DependencyFactory::default();
    let app = Application::new(
        "0.0.0.0:3000",
        ApplicationState::new(dependency_factory.sandbox_provider()),
    )
    .await?;
    app.run().await?;

    Ok(())
}
