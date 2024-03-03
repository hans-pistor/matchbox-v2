use std::{collections::HashMap, sync::Arc};

use axum::{routing::{get, post}, Router};
use tokio::{net::{TcpListener, ToSocketAddrs}, sync::RwLock};

use crate::sandbox::{Sandbox, SandboxFactory};

pub mod routes;

#[derive(Clone)]
pub struct ApplicationState(Arc<ApplicationStateInner>);

pub struct ApplicationStateInner {
    sandbox_factory: SandboxFactory,
    sandboxes: RwLock<HashMap<String, Sandbox>>
}

impl ApplicationState {
    pub fn new(sandbox_factory: SandboxFactory) -> Self {
        Self(Arc::new(ApplicationStateInner {
            sandbox_factory,
            sandboxes: Default::default(),
        }))
    }

    pub fn sandboxes(&self) -> &RwLock<HashMap<String, Sandbox>> {
        &self.0.sandboxes
    }

    pub fn sandbox_factory(&self) -> &SandboxFactory {
        &self.0.sandbox_factory
    }
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
        let listener = TcpListener::bind(address).await?;
        let router = Router::new()
            .route("/sandbox", get(routes::list_sandboxes))
            .route("/sandbox", post(routes::create_sandbox))
            .with_state(state);
        Ok(Application { listener, router })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.router).await?;

        Ok(())
    }
}
