use std::{collections::HashMap, sync::Arc};

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    signal,
    sync::RwLock,
};

use crate::sandbox::{ProvideSandbox, Sandbox};

pub mod routes;

#[derive(Clone)]
pub struct ApplicationState(Arc<ApplicationStateInner>);

pub struct ApplicationStateInner {
    sandbox_factory: Box<dyn ProvideSandbox + Send + Sync>,
    sandboxes: RwLock<HashMap<String, Sandbox>>,
}

impl ApplicationState {
    pub fn new(sandbox_factory: Box<dyn ProvideSandbox + Send + Sync>) -> Self {
        Self(Arc::new(ApplicationStateInner {
            sandbox_factory,
            sandboxes: Default::default(),
        }))
    }

    pub fn sandboxes(&self) -> &RwLock<HashMap<String, Sandbox>> {
        &self.0.sandboxes
    }

    #[allow(clippy::borrowed_box)]
    pub fn sandbox_factory(&self) -> &Box<dyn ProvideSandbox + Send + Sync> {
        &self.0.sandbox_factory
    }
}

pub struct Application {
    listener: TcpListener,
    router: Router,
}

impl Application {
    pub async fn new(address: impl ToSocketAddrs, state: ApplicationState) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(address).await?;
        let router = Router::new()
            .route("/sandbox", get(routes::sandbox::list::list_sandboxes))
            .route("/sandbox", post(routes::sandbox::create::create_sandbox))
            .route(
                "/sandbox/:id",
                delete(routes::sandbox::delete::delete_sandbox),
            )
            .with_state(state);
        Ok(Application { listener, router })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let shutdown_signal = async {
            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install ctrl-c handler")
            };

            tokio::select! {
                _ = ctrl_c => {}
            }
        };
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        Ok(())
    }
}
