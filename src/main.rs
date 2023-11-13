use crate::config::read_config;
use anyhow::Context;

use std::str::FromStr;
use time::Duration;

use hickory_client::{
    client::{Client, SyncClient},
    error::ClientErrorKind,
    op::{DnsResponse, ResponseCode},
    rr::{rdata::TXT, DNSClass, Name, RData, Record, RecordType},
    udp::UdpClientConnection,
};
use hickory_proto::error::ProtoErrorKind;

use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use once_cell::sync::OnceCell;

mod cli;
mod config;

static RESOLVER: OnceCell<std::net::SocketAddr> = OnceCell::new();

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = cli::parse();

    let config = read_config(&cli.config)?;
    RESOLVER
        .set(config.resolver)
        .map_err(|_| anyhow::format_err!("failed to set global recursor"))?;

    debug!("initializing with zones {:?}", config.zones.keys());

    let challenge_name: Name = find_record(&cli.identifier)?;
    info!("challenge name is {}", challenge_name);

    let zone: Name = find_zone(&challenge_name)?;
    info!("challenge zone is {}", zone);

    trace!("{:?}", config.zones);
    let primary_ns_client = &config
        .zones
        .get(&zone)
        .context("Couldn't find challenge zone in config, maybe you forgot the trailing dot?")?
        .create_client()?;

    let challenge_record = Record::from_rdata(
        challenge_name.clone(),
        u32::try_from(Duration::minutes(1).whole_seconds())?,
        RData::TXT(TXT::new(Vec::from([cli.proof]))),
    );

    match &cli.command {
        cli::Commands::Set => {
            let result = retry_op(primary_ns_client.create(challenge_record.clone(), zone))
                .context("Couldn't set record")?;
            match result.response_code() {
                ResponseCode::NoError => {
                    info!("record was successfully set");
                }
                ResponseCode::YXRRSet => {
                    warn!("{:?} already exists", challenge_record.clone());
                }
                _ => error!("unexpected response {:?}", result),
            }
            Ok(())
        }
        cli::Commands::Unset => {
            let result = retry_op(primary_ns_client.delete_rrset(challenge_record.clone(), zone))
                .context("Couldn't remove record")?;
            match result.response_code() {
                ResponseCode::NoError => {
                    info!("record was removed");
                }
                _ => error!("unexpected response {:?}", result),
            }
            Ok(())
        }
    }
}

fn find_record(identifier: &Name) -> anyhow::Result<Name> {
    let last_record = search(
        Name::from_str("_acme-challenge")?.append_name(identifier)?,
        RecordType::TXT,
    )?
    .answers()
    .last()
    .context("no record in response")?
    .clone()
    .into_parts();

    match last_record
        .rdata
        .context("record contains no answer section")?
    {
        RData::CNAME(name) => Ok(name.0),
        RData::TXT(_) => {
            info!(
                "found existing TXT record for {:?}",
                last_record.name_labels
            );
            Ok(last_record.name_labels)
        }
        _ => anyhow::bail!("unexpected record type"),
    }
}

fn find_zone(identifier: &Name) -> anyhow::Result<Name> {
    // Querries the SOA Record for the challenge idendifier.
    // This is based on the assumtion, it will always return a SOA Record.
    // I am unsure if this assumtion is correct.
    let response = search(identifier.clone(), RecordType::SOA)?;
    match response.response_code() {
        ResponseCode::NXDomain | ResponseCode::NoError => response
            .name_servers()
            .iter()
            .map(|record| record.clone().into_parts().name_labels)
            .last()
            .context("no zone found"),
        _ => Result::Err(anyhow::format_err!("unhandled reponse type")),
    }
}

pub fn search(identifier: Name, rtype: RecordType) -> anyhow::Result<DnsResponse> {
    let client = SyncClient::new(
        UdpClientConnection::new(
            *RESOLVER
                .get()
                .context("could not read resolver from config")?,
        )
        .context("failed to connect to recursor")?,
    );

    Ok(retry_op(client.query(&identifier, DNSClass::IN, rtype))?)
}

#[instrument]
pub fn retry_op(
    payload: Result<DnsResponse, hickory_client::error::ClientError>,
) -> Result<DnsResponse, backoff::Error<hickory_client::error::ClientError>> {
    backoff::retry(backoff::ExponentialBackoff::default(), || {
        match payload.clone() {
            Ok(response) => Ok(response),
            Err(e) => Result::Err({
                let transient = backoff::Error::Transient {
                    err: e.clone(),
                    retry_after: None,
                };
                let permanent = backoff::Error::Permanent(e.clone());

                match &e.kind() {
                    ClientErrorKind::Proto(proto_error) => match proto_error.kind() {
                        ProtoErrorKind::Busy | ProtoErrorKind::Io(_) | ProtoErrorKind::Timeout => {
                            transient
                        }
                        _ => permanent,
                    },
                    ClientErrorKind::SendError(_)
                    | ClientErrorKind::Io(_)
                    | ClientErrorKind::Timeout => transient,
                    _ => permanent,
                }
            }),
        }
    })
}
