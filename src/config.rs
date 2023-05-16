use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, MapPreventDuplicates, DisplayFromStr, base64::Base64};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::str::FromStr;
use trust_dns_client::rr::Name;
use trust_dns_client::rr::rdata::tsig::{TsigAlgorithm};

#[serde_as]
#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) resolver: std::net::SocketAddr,
    #[serde_as(as = "MapPreventDuplicates<DisplayFromStr, _>")]
    pub(crate) zones: HashMap<Name, ZoneConfig>,
}

#[serde_as]
#[derive(Deserialize)]
pub(crate) struct ZoneConfig {
    pub(crate) primary_ns: std::net::SocketAddr,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) tsig_name: Name,
    #[serde_as(as = "Base64")]
    pub(crate) tsig_key: Vec<u8>,
    #[serde(deserialize_with = "deserialize_tsig_algorithm")]
    pub(crate) tsig_algorithm: TsigAlgorithm,
}

pub(crate) fn read_config(path: &std::path::Path) -> Result<Config> {
    // TODO: error message on config not found
    let config_file_content = std::fs::read_to_string(path).context("Couldn't read config file")?;
    toml::from_str(&config_file_content).context("Couldn't parse config file")
}

fn deserialize_tsig_algorithm<'de, D>(
    deserializer: D
) -> Result<TsigAlgorithm, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let name = Name::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(TsigAlgorithm::from_name(name))
}
