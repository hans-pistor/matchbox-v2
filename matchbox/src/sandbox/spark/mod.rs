use anyhow::Context;
use sparklib::grpc::{
    guest_agent_client::GuestAgentClient, HealthCheckRequest, HealthCheckResponse, MountRequest,
    MountResponse,
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

    pub async fn mount_drive(
        &mut self,
        device: String,
        path: String,
    ) -> anyhow::Result<MountResponse> {
        let request = Request::new(MountRequest { device, path });
        self.client
            .mount(request)
            .await
            .map(|r| r.into_inner())
            .context("failed to mount drive in uVM")
    }
}
