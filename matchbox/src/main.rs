use matchbox::dependency::DependencyFactory;




use matchbox::server::{Application, ApplicationState};



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
