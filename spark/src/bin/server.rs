use spark::grpc::guest_agent_server::{GuestAgent, GuestAgentServer};
use spark::grpc::{HealthCheckRequest, HealthCheckResponse};
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

#[tonic::async_trait]
impl GuestAgent for SparkServer {
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {}))
    }
}
