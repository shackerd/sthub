use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub network: Option<NetworkConfiguration>,
    pub hubs: Option<ConfigurationHubs>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigurationHubs {
    #[serde(alias = "static")]
    pub _static: Option<StaticHubConfiguration>,
    pub configuration: Option<ConfigurationHubConfiguration>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigurationHubConfiguration {
    pub host: Option<String>,
    pub cache: Option<bool>,
    pub providers: Option<ConfigurationHubProviders>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigurationHubProviders {
    pub env: Option<EnvConfigurationHubProvider>,
    pub dotenv: Option<DotenvConfigurationProvider>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfigurationHubProvider {
    pub prefix: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DotenvConfigurationProvider {
    pub hotreload: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StaticHubConfiguration {
    pub path: Option<String>,
    pub host: Option<String>,
    pub rewrite_rules: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkConfiguration {
    pub port: Option<u16>,
}

pub async fn load_configuration(
    path: &str,
) -> Result<Configuration, Box<dyn std::error::Error + Send + Sync>> {
    let res = tokio::fs::read_to_string(path)
        .await
        .expect("Failed to read configuration file");

    let config = serde_yaml::from_str::<Configuration>(&res)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_configuration() {
        let config = load_configuration("config.yaml").await;
        assert!(config.is_ok());
        assert_eq!(
            config
                .unwrap()
                .hubs
                .unwrap()
                .configuration
                .unwrap()
                .providers
                .unwrap()
                .env
                .unwrap()
                .prefix
                .unwrap(),
            "STHUB__"
        );
    }
}
