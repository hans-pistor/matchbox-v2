use anyhow::Context;
use sparklib::grpc::{
    guest_agent_client::GuestAgentClient, HealthCheckRequest, HealthCheckResponse,
};
use tonic::{
    transport::{Channel, Endpoint},
    Request,
};

pub mod factory;

#[derive(Debug, Clone)]
pub struct SparkClient {
    client: GuestAgentClient<Channel>,
}

impl SparkClient {
    pub async fn initialize(ip: &str) -> anyhow::Result<SparkClient> {
        let address = format!("http://{ip}:5001");
        let channel = Endpoint::new(address)?.connect_lazy();
        let client = GuestAgentClient::new(channel);
        Ok(SparkClient { client })
    }

    pub async fn health_check(&mut self) -> anyhow::Result<HealthCheckResponse> {
        let request = Request::new(HealthCheckRequest {});
        self.client
            .health_check(request)
            .await
            .map(|r| r.into_inner())
            .context("health check failed for uVM")
    }
}
