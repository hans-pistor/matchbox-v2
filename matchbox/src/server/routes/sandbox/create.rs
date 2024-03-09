use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    sandbox::{Location, ProvideSandboxOptionsBuilder},
    server::{routes::ApiResult, ApplicationState},
};

use super::SandboxResponse;

#[derive(Serialize, Deserialize)]
pub struct CreateSandboxRequest {
    code_drive_path: Location,
}

#[axum_macros::debug_handler]
pub async fn create_sandbox(
    State(state): State<ApplicationState>,
    Json(payload): Json<CreateSandboxRequest>,
) -> ApiResult<SandboxResponse> {
    let factory = state.sandbox_factory();
    let options = ProvideSandboxOptionsBuilder::default()
        .code_drive_location(payload.code_drive_path)
        .build()?;
    let sandbox = factory.provide_sandbox(options).await?;

    let response = SandboxResponse::from(&sandbox);
    {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.insert(sandbox.id().to_string(), sandbox);
    }
    Ok(response)
}
