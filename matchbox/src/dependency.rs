use std::sync::Arc;

use crate::{
    jailer::factory::{JailedFirecrackerFactory, ProvideFirecracker},
    sandbox::{
        id::{ProvideIdentifier, VmIdentifierFactory},
        spark::factory::{ProvideSparkClient, SparkClientFactory},
        InitializeSandbox, ProvideSandbox, SandboxFactory, SandboxInitializer,
    },
};

pub struct DependencyFactory {
    firecracker_provider: Arc<Box<dyn ProvideFirecracker>>,
    sandbox_initialixer: Arc<Box<dyn InitializeSandbox>>,
    identifier_provider: Arc<Box<dyn ProvideIdentifier>>,
    spark_client_provider: Arc<Box<dyn ProvideSparkClient>>,
}

impl DependencyFactory {
    pub fn sandbox_provider(&self) -> Box<dyn ProvideSandbox + Send + Sync> {
        let sandbox_provider = SandboxFactory::new(
            self.identifier_provider.clone(),
            self.spark_client_provider.clone(),
            self.firecracker_provider.clone(),
            self.sandbox_initialixer.clone(),
        );
        Box::new(sandbox_provider)
    }

    pub fn with_firecracker_provider(
        self,
        firecracker_provider: Arc<Box<dyn ProvideFirecracker>>,
    ) -> Self {
        Self {
            firecracker_provider,
            ..self
        }
    }

    pub fn with_spark_client_provider(
        self,
        spark_client_provider: Arc<Box<dyn ProvideSparkClient>>,
    ) -> Self {
        Self {
            spark_client_provider,
            ..self
        }
    }

    pub fn with_identifier_provider(
        self,
        identifier_provider: Arc<Box<dyn ProvideIdentifier>>,
    ) -> Self {
        Self {
            identifier_provider,
            ..self
        }
    }

    pub fn with_sandbox_initializer(
        self,
        sandbox_initialixer: Arc<Box<dyn InitializeSandbox>>,
    ) -> Self {
        Self {
            sandbox_initialixer,
            ..self
        }
    }
}

impl Default for DependencyFactory {
    fn default() -> Self {
        let firecracker_provider: Box<dyn ProvideFirecracker> =
            Box::new(JailedFirecrackerFactory::new(
                "/usr/local/bin/jailer",
                "/usr/local/bin/firecracker",
                "/tmp/vms",
            ));
        let sandbox_initializer: Box<dyn InitializeSandbox> = Box::new(SandboxInitializer::new(
            "/tmp/rootfs.ext4",
            "/tmp/kernel.bin",
        ));
        let identifier_provider: Box<dyn ProvideIdentifier> =
            Box::new(VmIdentifierFactory::default());
        let spark_client_provider: Box<dyn ProvideSparkClient> =
            Box::new(SparkClientFactory::default());
        Self {
            firecracker_provider: Arc::from(firecracker_provider),
            sandbox_initialixer: Arc::from(sandbox_initializer),
            identifier_provider: Arc::from(identifier_provider),
            spark_client_provider: Arc::from(spark_client_provider),
        }
    }
}
