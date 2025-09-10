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

    pub fn init_dotenv(&self) -> Result<(), std::io::Error> {
        let conf = self
            .configuration
            .hubs
            .as_ref()
            .and_then(|f| f.configuration.as_ref())
            .and_then(|f| f.providers.clone())
            .and_then(|f| f.dotenv);

        if conf.as_ref().is_none() {
            println!("No dotenv configuration found, skipping dotenv initialization.");
            return Ok(());
        }

        let conf = conf.unwrap();

        if let Some(path) = conf.path {
            println!("Loading .env from path: {path}");
            dotenvy::from_filename(path).ok();
        } else {
            println!("Loading .env from default path: .env");
            dotenvy::from_filename(".env").ok();
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod test {}
