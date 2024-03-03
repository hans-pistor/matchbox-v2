use std::path::PathBuf;

use anyhow::Context;
use hyper::{Body, Client, Method, Request, Response};
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct FirecrackerClient {
    client: Client<UnixConnector>,
    socket_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "PascalCase")]
pub enum Action {
    InstanceStart,
}

impl FirecrackerClient {
    pub fn new(socket_path: PathBuf) -> FirecrackerClient {
        let client = Client::unix();
        Self {
            client,
            socket_path,
        }
    }

    pub async fn get(&self, path: impl AsRef<str>) -> anyhow::Result<Response<Body>> {
        self.execute(path, Method::GET, Body::empty()).await
    }

    pub async fn put(
        &self,
        path: impl AsRef<str>,
        body: impl Serialize,
    ) -> anyhow::Result<Response<Body>> {
        let body = Body::from(serde_json::to_string(&body)?);
        self.execute(path, Method::PUT, body).await
    }

    pub async fn action(&self, action: Action) -> anyhow::Result<Response<Body>> {
        self.put("/actions", &action).await
    }

    async fn execute(
        &self,
        path: impl AsRef<str>,
        method: Method,
        body: Body,
    ) -> anyhow::Result<Response<Body>> {
        let uri: hyper::Uri = Uri::new(&self.socket_path, path.as_ref()).into();

        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Acccept", "application/json")
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap();

        self.client
            .request(request)
            .await
            .context("Failed to execute request")
    }
}
