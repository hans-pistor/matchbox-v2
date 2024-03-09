use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    server::{routes::ApiResult, ApplicationState},
};

#[derive(Serialize, Deserialize)]
pub struct ExecuteResponse {
    output: String,
}

impl IntoResponse for ExecuteResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[axum_macros::debug_handler]
pub async fn execute_sandbox(
    Path(sandbox_id): Path<String>,
    State(state): State<ApplicationState>,
) -> ApiResult<ExecuteResponse> {
    let sandboxes = state.sandboxes().read().await;
    let sandbox = match sandboxes.get(&sandbox_id) {
        Some(s) => s,
        None => {
            return Err(
                anyhow::Error::msg(format!("Sandbox with id {sandbox_id} was not found")).into(),
            )
        }
    };
    let mut client = sandbox.client().await;
    let response = client
        .execute(
            "python3".to_string(),
            ["/tmp/vdb/entrypoint.py"].map(String::from).to_vec(),
        )
        .await?;
    Ok(ExecuteResponse {
        output: response.output,
    })
}
