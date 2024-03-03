use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use super::ApplicationState;

pub type ApiResult<T> = Result<T, error::ApiError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListSandboxesResponse {
    sandboxes: Vec<String>,
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
    let sandboxes = sandboxes.keys().cloned().collect::<Vec<String>>();
    Ok(ListSandboxesResponse { sandboxes })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxDetailResponse {
    id: String,
}

impl IntoResponse for SandboxDetailResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub async fn create_sandbox(
    State(state): State<ApplicationState>,
) -> ApiResult<SandboxDetailResponse> {
    let factory = state.sandbox_factory();
    let mut sandbox = factory.spawn_sandbox().await?;
    sandbox.start().await?;
    let id = sandbox.id().to_string();
    {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.insert(id.clone(), sandbox);
    }
    Ok(SandboxDetailResponse { id })
}

pub async fn delete_sandbox(
    Path(sandbox_id): Path<String>,
    State(state): State<ApplicationState>,
) -> ApiResult<SandboxDetailResponse> {
    let sandbox = {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.remove(&sandbox_id)
    };

    match sandbox {
        Some(sandbox) => Ok(SandboxDetailResponse {
            id: sandbox.id().to_string(),
        }),
        None => Err(anyhow::anyhow!("Sandbox with id {sandbox_id} does not exist").into()),
    }
}

mod error {
    pub struct ApiError(anyhow::Error);
    impl axum::response::IntoResponse for ApiError {
        fn into_response(self) -> axum::response::Response {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", self.0),
            )
                .into_response()
        }
    }

    impl<E> From<E> for ApiError
    where
        E: Into<anyhow::Error>,
    {
        fn from(err: E) -> Self {
            Self(err.into())
        }
    }
}
