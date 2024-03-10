#![allow(dead_code)]
use std::{
    process::Command,
    time::{Duration, Instant},
};

use matchbox::{
    dependency::DependencyFactory,
    server::{
        routes::sandbox::{create::CreateSandboxRequest, SandboxResponse},
        Application, ApplicationState,
    },
};
use tokio::{net::TcpListener, task::JoinHandle};

pub struct TestServer {
    address: String,
    handle: JoinHandle<()>,
}

impl TestServer {
    pub async fn default() -> TestServer {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let dependency = DependencyFactory::default();
        let state = ApplicationState::new(dependency.sandbox_provider());
        let application = Application::new(format!("127.0.0.1:{port}"), state)
            .await
            .unwrap();
        let handle = tokio::spawn(async {
            application.run().await.unwrap();
        });
        let address = format!("http://127.0.0.1:{port}");

        TestServer { address, handle }
    }

    pub async fn create_vm(&self, request: CreateSandboxRequest) -> SandboxResponse {
        reqwest::Client::new()
            .post(format!("{}/sandbox", self.address))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
            .expect("failed to send the create vm request")
            .text()
            .await
            .map(|body| serde_json::from_str::<SandboxResponse>(&body).unwrap())
            .expect("failed to get or deserialize response")
    }
}

pub fn ping(ip_address: impl AsRef<str>) -> anyhow::Result<()> {
    let mut cmd = Command::new("timeout");
    let output = cmd
        .args(["1", "ping", "-c", "1", ip_address.as_ref()])
        .output()?;

    match output.status.success() {
        true => Ok(()),
        false => anyhow::bail!(
            "failed to ping the {}. {}",
            ip_address.as_ref(),
            String::from_utf8_lossy(&output.stdout)
        ),
    }
}

pub fn wait_until(duration: Duration, func: impl Fn() -> anyhow::Result<()>) -> anyhow::Result<()> {
    let start = Instant::now();
    while Instant::now() < start + duration {
        match func() {
            Ok(_) => return Ok(()),
            Err(e) => println!("got error from function: {e:?}"),
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    anyhow::bail!("function did not return true after {duration:?}");
}
