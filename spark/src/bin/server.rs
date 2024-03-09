use std::fs::create_dir_all;
use std::process::Command;

use sparklib::grpc::guest_agent_server::{GuestAgent, GuestAgentServer};
use sparklib::grpc::{HealthCheckRequest, HealthCheckResponse, MountRequest, MountResponse};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let spark_server = SparkServer::default();

    Server::builder()
        .add_service(GuestAgentServer::new(spark_server))
        .serve("0.0.0.0:5001".parse()?)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct SparkServer {}

impl SparkServer {
    pub fn handle_mount_request(&self, request: &MountRequest) -> anyhow::Result<()> {
        std::fs::create_dir_all(&request.path)?;
        let mut cmd = Command::new("mount");
        let output = cmd.args([&request.device, &request.path]).output()?;
        if !output.status.success() {
            anyhow::bail!(
                "mount command failed: {}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl GuestAgent for SparkServer {
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {}))
    }

    async fn mount(
        &self,
        request: Request<MountRequest>,
    ) -> Result<Response<MountResponse>, Status> {
        self.handle_mount_request(request.get_ref())
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(MountResponse {}))
    }
}
