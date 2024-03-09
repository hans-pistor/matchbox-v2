use sparklib::grpc::guest_agent_client::GuestAgentClient;
use sparklib::grpc::HealthCheckRequest;
use tonic::Request;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let mut client = GuestAgentClient::connect("http://localhost:5001").await?;
    let request = Request::new(HealthCheckRequest {});
    let response = client.health_check(request).await?;

    println!("{response:?}");
    Ok(())
}
