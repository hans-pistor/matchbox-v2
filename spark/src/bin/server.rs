use std::process::Command;

use sparklib::grpc::guest_agent_server::{GuestAgent, GuestAgentServer};
use sparklib::grpc::{
    ExecuteRequest, ExecuteResponse, HealthCheckRequest, HealthCheckResponse, MountRequest,
    MountResponse,
};
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

    pub fn handle_execute_request(
        &self,
        request: &ExecuteRequest,
    ) -> anyhow::Result<ExecuteResponse> {
        std::env::set_current_dir("/tmp/vdb")?;
        let mut cmd = Command::new(&request.command);
        let output = cmd.args(&request.arguments).output()?;
        if !output.status.success() {
            anyhow::bail!(
                "execute {} {} failed: {}\n{}",
                request.command,
                request.arguments.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(ExecuteResponse {
            output: String::from_utf8_lossy(&output.stdout).to_string(),
        })
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

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        self.handle_execute_request(request.get_ref())
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
