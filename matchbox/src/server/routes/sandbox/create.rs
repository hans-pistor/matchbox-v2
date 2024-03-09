use axum::extract::State;

use crate::{
    sandbox::ProvideSandboxOptionsBuilder,
    server::{routes::ApiResult, ApplicationState},
};

use super::SandboxResponse;

pub async fn create_sandbox(State(state): State<ApplicationState>) -> ApiResult<SandboxResponse> {
    let factory = state.sandbox_factory();
    let options = ProvideSandboxOptionsBuilder::default().build()?;
    let sandbox = factory.provide_sandbox(options).await?;

    let response = SandboxResponse::from(&sandbox);
    {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.insert(sandbox.id().to_string(), sandbox);
    }
    Ok(response)
}
