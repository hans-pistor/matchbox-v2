use jailer::factory::JailedFirecrackerFactory;
use sandbox::SandboxFactory;
use server::{Application, ApplicationState};

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
    let sandbox_factory = SandboxFactory::new(factory, sandbox_initializer);

    let app = Application::new("0.0.0.0:3000", ApplicationState { sandbox_factory }).await?;
    app.run().await?;

    Ok(())
}

pub mod server {

    use std::sync::Arc;

    use axum::{routing::get, Router};
    use tokio::net::{TcpListener, ToSocketAddrs};

    use crate::sandbox::SandboxFactory;

    pub mod routes {
        use axum::response::IntoResponse;

        pub async fn root() -> impl IntoResponse {
            "root route"
        }
    }

    #[derive(Clone)]
    pub struct ApplicationState {
        pub sandbox_factory: SandboxFactory,
    }

    pub struct Application {
        listener: TcpListener,
        router: Router,
    }

    impl Application {
        pub async fn new(
            address: impl ToSocketAddrs,
            state: ApplicationState,
        ) -> anyhow::Result<Self> {
            let state = Arc::new(state);
            let listener = TcpListener::bind(address).await?;
            let router = Router::new()
                .route("/", get(routes::root))
                .with_state(state);
            Ok(Application { listener, router })
        }

        pub async fn run(self) -> anyhow::Result<()> {
            axum::serve(self.listener, self.router).await?;

            Ok(())
        }
    }
}
