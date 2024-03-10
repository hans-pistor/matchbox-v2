use matchbox::{
    sandbox::spark::SparkClient, server::routes::sandbox::create::CreateSandboxRequest,
};

use crate::common::{ping, TestServer};

#[tokio::test]
async fn test_create_sandbox_with_no_code_drive_success() {
    let server = TestServer::default().await;
    let response = server
        .create_vm(CreateSandboxRequest {
            code_drive_path: None,
        })
        .await;

    ping(&response.ip).expect("We should be able to ping the guest");

    let mut client = SparkClient::initialize(&response.ip).await.unwrap();

    assert!(
        client.health_check().await.is_ok(),
        "Spark health check guaranteed to succced since uVM booted"
    );

    let output = client
        .execute(
            "ping".into(),
            ["-c", "1", "8.8.8.8"].map(String::from).to_vec(),
        )
        .await
        .unwrap()
        .output;

    assert!(
        output.contains("1 packets received"),
        "We should be able to connect to the internet from the uVM"
    );
}
