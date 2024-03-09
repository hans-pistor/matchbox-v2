use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::sandbox::Sandbox;

use super::ApplicationState;

pub type ApiResult<T> = Result<T, error::ApiError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct SandboxResponse {
    id: String,
    ip: String,
}

impl From<&Sandbox> for SandboxResponse {
    fn from(value: &Sandbox) -> Self {
        SandboxResponse {
            id: value.id().to_string(),
            ip: value.network().microvm_ip(),
        }
    }
}

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

impl IntoResponse for SandboxResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub async fn create_sandbox(State(state): State<ApplicationState>) -> ApiResult<SandboxResponse> {
    let factory = state.sandbox_factory();
    let sandbox = factory.provide_sandbox().await?;

    let response = SandboxResponse::from(&sandbox);
    {
        let mut sandboxes = state.sandboxes().write().await;
        sandboxes.insert(sandbox.id().to_string(), sandbox);
    }
    Ok(response)
}

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
