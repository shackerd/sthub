use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub network: Option<NetworkConfiguration>,
    pub global: Option<GlobalConfiguration>,
    pub hubs: Option<ConfigurationHubs>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigurationHubs {
    #[serde(alias = "static")]
    pub _static: Option<StaticHubConfiguration>,
    pub configuration: Option<ConfigurationHubConfiguration>,
    pub upstream: Option<UpstreamConfiguration>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigurationHubConfiguration {
    pub remote_path: Option<String>,
    pub cache: Option<bool>,
    pub headers: Option<HashMap<String, String>>,
    pub providers: Option<ConfigurationHubProviders>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GlobalConfiguration {
    pub headers: Option<HashMap<String, String>>,
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
    pub remote_path: Option<String>,
    pub path: Option<String>,
    pub rewrite_rules: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpstreamConfiguration {
    pub target: Option<String>,
    pub remote_path: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkConfiguration {
    pub port: Option<u16>,
    pub host: Option<String>,
}

pub async fn load_configuration(path: &str) -> std::io::Result<Configuration> {
    let res = tokio::fs::read_to_string(path)
        .await
        .expect("Failed to read configuration file");

    let config = serde_yaml::from_str::<Configuration>(&res).unwrap();
    Ok(config)
}

#[cfg(test)]
mod tests {

    use assertables::assert_starts_with;

    use super::*;

    #[tokio::test]
    async fn test_load_configuration() {
        let conf = load_configuration("conf.yaml").await;
        assert!(conf.is_ok());
        assert_eq!(
            conf.unwrap()
                .hubs
                .and_then(|c| c.configuration)
                .and_then(|c| c.providers)
                .and_then(|c| c.env)
                .and_then(|c| c.prefix)
                .unwrap_or("KO".to_string()),
            "STHUB__"
        );
    }
}
