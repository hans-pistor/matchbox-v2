use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::server::{routes::ApiResult, ApplicationState};

use super::SandboxResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListSandboxesResponse {
    sandboxes: Vec<SandboxResponse>,
}

impl IntoResponse for ListSandboxesResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub async fn list_sandboxes(
    State(state): State<ApplicationState>,
) -> ApiResult<ListSandboxesResponse> {
    let sandboxes = state.sandboxes().read().await;
    let sandboxes = sandboxes
        .iter()
        .map(|(_, sb)| SandboxResponse::from(sb))
        .collect();
    Ok(ListSandboxesResponse { sandboxes })
}
