use anyhow::{Context, Result};
use hickory_client::{
    client::SyncClient,
    rr::{rdata::tsig::TsigAlgorithm, Name},
    udp::UdpClientConnection,
};
use hickory_proto::rr::dnssec::tsig::TSigner;
use serde::{Deserialize, Deserializer};
use serde_with::{base64::Base64, serde_as, DisplayFromStr, MapPreventDuplicates};
use std::{collections::HashMap, str::FromStr};

#[serde_as]
#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) resolver: std::net::SocketAddr,
    #[serde_as(as = "MapPreventDuplicates<DisplayFromStr, _>")]
    pub(crate) zones: HashMap<Name, ZoneConfig>,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub(crate) struct ZoneConfig {
    pub(crate) primary_ns: std::net::SocketAddr,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) tsig_name: Name,
    #[serde_as(as = "Base64")]
    pub(crate) tsig_key: Vec<u8>,
    #[serde(deserialize_with = "deserialize_tsig_algorithm")]
    pub(crate) tsig_algorithm: TsigAlgorithm,
}

impl ZoneConfig {
    pub fn create_client(&self) -> Result<SyncClient<UdpClientConnection>> {
        Ok(SyncClient::with_tsigner(
            UdpClientConnection::new(self.primary_ns)
                .context("failed to establish connection to primary ns")?,
            TSigner::new(
                self.tsig_key.clone(),
                self.tsig_algorithm.clone(),
                self.tsig_name.clone(),
                300,
            )
            .context("failed to create tsigner")?,
        ))
    }
}

pub(crate) fn read_config(path: &std::path::Path) -> Result<Config> {
    // TODO: error message on config not found
    let config_file_content = std::fs::read_to_string(path).context("Couldn't read config file")?;
    toml::from_str(&config_file_content).context("Couldn't parse config file")
}

fn deserialize_tsig_algorithm<'de, D>(deserializer: D) -> Result<TsigAlgorithm, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let name = Name::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(TsigAlgorithm::from_name(name))
}
