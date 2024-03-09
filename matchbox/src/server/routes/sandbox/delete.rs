use axum::extract::{Path, State};

use crate::server::{routes::ApiResult, ApplicationState};

use super::SandboxResponse;

pub async fn delete_sandbox(
    Path(sandbox_id): Path<String>,
    State(state): State<ApplicationState>,
) -> ApiResult<SandboxResponse> {
    let sandbox = {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.remove(&sandbox_id)
    };

    match sandbox {
        Some(sandbox) => Ok(SandboxResponse::from(&sandbox)),
        None => Err(anyhow::anyhow!("Sandbox with id {sandbox_id} does not exist").into()),
    }
}
