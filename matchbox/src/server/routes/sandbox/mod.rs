use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::sandbox::Sandbox;

pub mod create;
pub mod delete;
pub mod execute;
pub mod list;

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

impl IntoResponse for SandboxResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
