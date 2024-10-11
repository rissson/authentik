use std::{net::SocketAddr, sync::LazyLock};

use anyhow::{Result, anyhow};
use config;
use serde::{Deserialize, Serialize};
use url::Url;

pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| Settings::new().unwrap());

fn parse_uri(value: String) -> Result<String> {
    match Url::parse(&value) {
        Ok(url) => match url.scheme() {
            "env" => match std::env::var(value) {
                Ok(v) => Ok(v),
                _ => Ok(url.query().unwrap().to_owned()),
            },
            "file" => Ok(std::fs::read_to_string(url.host_str().unwrap())?),
            _ => Ok(value),
        },
        _ => Ok(value),
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub listen: ListenSettings,
    pub log_level: tracing::Level,
    pub debug: bool,
    pub error_reporting: ErrorReportingSettings,
    pub storage: StorageSettings,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct SettingsRaw {
    listen: ListenSettingsRaw,
    log_level: String,
    debug: String,
    error_reporting: ErrorReportingSettingsRaw,
    storage: StorageSettingsRaw,
}

impl TryInto<Settings> for SettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Settings> {
        Ok(Settings {
            listen: self.listen.try_into()?,
            log_level: parse_uri(self.log_level)?
                .parse()
                .map_err(|e| anyhow!("Failed to parse log level: {e}"))?,
            debug: parse_uri(self.debug)?.parse()?,
            error_reporting: self.error_reporting.try_into()?,
            storage: self.storage.try_into()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ListenSettings {
    pub http: SocketAddr,
    pub https: SocketAddr,
    pub metrics: SocketAddr,
    pub trusted_proxy_cidrs: Vec<ipnet::IpNet>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct ListenSettingsRaw {
    #[serde(alias = "listen_http")]
    http: String,
    #[serde(alias = "listen_https")]
    https: String,
    #[serde(alias = "listen_metrics")]
    metrics: String,
    trusted_proxy_cidrs: Vec<String>,
}

impl TryInto<ListenSettings> for ListenSettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ListenSettings> {
        let mut trusted_proxy_cidrs = vec![];
        for trusted_proxy_cidr in self.trusted_proxy_cidrs.iter() {
            trusted_proxy_cidrs.push(parse_uri(trusted_proxy_cidr.to_string())?.parse()?);
        }
        Ok(ListenSettings {
            http: parse_uri(self.http)?.parse()?,
            https: parse_uri(self.https)?.parse()?,
            metrics: parse_uri(self.metrics)?.parse()?,
            trusted_proxy_cidrs,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ErrorReportingSettings {
    pub enabled: bool,
    pub sentry_dsn: String,
    pub environment: String,
    pub send_pii: bool,
    pub sample_rate: f32,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
struct ErrorReportingSettingsRaw {
    enabled: String,
    sentry_dsn: String,
    environment: String,
    send_pii: String,
    sample_rate: String,
}

impl TryInto<ErrorReportingSettings> for ErrorReportingSettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ErrorReportingSettings> {
        Ok(ErrorReportingSettings {
            enabled: parse_uri(self.enabled)?.parse()?,
            sentry_dsn: parse_uri(self.sentry_dsn)?,
            environment: parse_uri(self.environment)?,
            send_pii: parse_uri(self.send_pii)?.parse()?,
            sample_rate: parse_uri(self.sample_rate)?.parse()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StorageSettings {
    pub media: StorageMediaSettings,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
struct StorageSettingsRaw {
    media: StorageMediaSettingsRaw,
}

impl TryInto<StorageSettings> for StorageSettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StorageSettings> {
        Ok(StorageSettings {
            media: self.media.try_into()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StorageMediaSettings {
    pub backend: String, // TODO: make this an enum
    pub file: StorageMediaFileSettings,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
struct StorageMediaSettingsRaw {
    backend: String,
    file: StorageMediaFileSettingsRaw,
}

impl TryInto<StorageMediaSettings> for StorageMediaSettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StorageMediaSettings> {
        Ok(StorageMediaSettings {
            backend: parse_uri(self.backend)?,
            file: self.file.try_into()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StorageMediaFileSettings {
    pub path: String,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
struct StorageMediaFileSettingsRaw {
    path: String,
}

impl TryInto<StorageMediaFileSettings> for StorageMediaFileSettingsRaw {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StorageMediaFileSettings> {
        Ok(StorageMediaFileSettings {
            path: parse_uri(self.path)?,
        })
    }
}

impl Settings {
    // TODO: add hot reload
    fn new() -> Result<Self> {
        let environment = match std::env::var("AUTHENTIK_ENV") {
            Ok(env) => env,
            _ => "local".to_owned(),
        };
        let config_builder = config::Config::builder()
            .add_source(config::File::from_str(
                include_str!("../../../authentik/lib/default.yml"),
                config::FileFormat::Yaml,
            ))
            .add_source(config::File::with_name("/etc/authentik/config.yml").required(false))
            .add_source(config::File::with_name(&format!("{environment}.yml")).required(false))
            .add_source(config::File::with_name(&format!("{environment}.env.yml")).required(false))
            .add_source(config::File::with_name(&format!("{environment}.yaml")).required(false))
            .add_source(config::File::with_name(&format!("{environment}.env.yaml")).required(false))
            .add_source(
                glob::glob("/etc/authentik/config.d/*.yml")?
                    .map(|path| config::File::from(path.unwrap()))
                    .collect::<Vec<_>>(),
            )
            .add_source(
                config::Environment::with_prefix("authentik")
                    .prefix_separator("_")
                    .separator("__"),
            );

        let config = config_builder.build()?;
        let raw: SettingsRaw = config.try_deserialize()?;

        raw.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() -> Result<()> {
        let _settings = Settings::new()?;
        Ok(())
    }
}
