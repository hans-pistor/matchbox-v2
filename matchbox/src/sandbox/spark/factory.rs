use std::fmt::Debug;

use super::SparkClient;

#[async_trait::async_trait]
pub trait ProvideSparkClient: Debug + Send + Sync {
    async fn provide_spark_client(&self, address: &str) -> anyhow::Result<SparkClient>;
}

#[derive(Debug)]
pub struct SparkClientFactory;

#[async_trait::async_trait]
impl ProvideSparkClient for SparkClientFactory {
    async fn provide_spark_client(&self, address: &str) -> anyhow::Result<SparkClient> {
        SparkClient::initialize(address).await
    }
}
