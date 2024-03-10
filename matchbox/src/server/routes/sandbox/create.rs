use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    sandbox::{Location, ProvideSandboxOptionsBuilder},
    server::{routes::ApiResult, ApplicationState},
};

use super::SandboxResponse;

#[derive(Serialize, Deserialize)]
pub struct CreateSandboxRequest {
    pub code_drive_path: Option<Location>,
}

#[axum_macros::debug_handler]
pub async fn create_sandbox(
    State(state): State<ApplicationState>,
    Json(payload): Json<CreateSandboxRequest>,
) -> ApiResult<SandboxResponse> {
    let factory = state.sandbox_factory();
    let mut builder = ProvideSandboxOptionsBuilder::default();
    if let Some(path) = payload.code_drive_path {
        builder.code_drive_location(path);
    }
    let sandbox = factory.provide_sandbox(builder.build()?).await?;

    let response = SandboxResponse::from(&sandbox);
    {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.insert(sandbox.id().to_string(), sandbox);
    }
    Ok(response)
}
