use crate::{
    core::{
        cli,
        configuration::{self, Configuration},
    },
    net::http_adapter::HttpAdapter,
};
use clap::Parser;

/// boot up the application kernel
/// ``` rust
/// let krn = kernel::boot().await?;
/// ```
pub async fn boot() -> std::io::Result<Kernel> {
    let cli = cli::Cli::parse();

    let conf = configuration::load_configuration(
        &cli.configuration_path.unwrap_or("conf.yaml".to_string()),
    )
    .await
    .unwrap();

    Ok(Kernel::new(conf))
}

/// The application kernel, responsible for managing the application's lifecycle and providing access to its core components.
pub struct Kernel {
    configuration: Configuration,
}

impl Kernel {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }

    pub fn setup_http_adapter(&self) -> HttpAdapter {
        HttpAdapter::new(&self.configuration)
    }
}

#[cfg(test)]
pub mod test {}
