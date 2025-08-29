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

pub async fn load_configuration(path: &str) -> std::io::Result<Configuration> {
    let res = tokio::fs::read_to_string(path)
        .await
        .expect("Failed to read configuration file");

    let config = serde_yaml::from_str::<Configuration>(&res).unwrap();
    Ok(config)
}

pub fn try_get_rules(conf: &Configuration) -> Option<String> {
    if let Some(h) = conf.hubs.clone() {
        if let Some(s) = h._static {
            return s.rewrite_rules;
        }
    }
    None
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

    #[tokio::test]
    async fn test_try_get_rules() {
        let conf = load_configuration("conf.yaml").await;
        assert!(conf.is_ok());

        let conf = conf.unwrap();

        assert_starts_with!(try_get_rules(&conf).unwrap(), "RewriteEngine On");
    }
}
