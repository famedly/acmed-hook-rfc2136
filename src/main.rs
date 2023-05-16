use anyhow::{Context};
use clap::Parser;
use std::str::FromStr;
use time::Duration;
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::udp::UdpClientConnection;
use trust_dns_client::op::DnsResponse;
use trust_dns_client::rr::dnssec::tsig::{TSigner};
use trust_dns_client::rr::rdata::{TXT};
use trust_dns_client::rr::{Name, RData, Record, RecordType};
use crate::config::{read_config};

mod config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    config: std::path::PathBuf,
    #[arg(long)]
    identifier: String,
    #[arg(long)]
    proof: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = read_config(&cli.config)?;
    let mut record = find_record(&cli.identifier)?;
    let origin = record.name().clone(); // TODO: find out zone via SOA

    let zone_config = &config.zones.get(record.name()).unwrap(); // TODO: Handle None
    let tsigner = TSigner::new(zone_config.tsig_key.clone(),
                               zone_config.tsig_algorithm.clone(),
                               zone_config.tsig_name.clone(), 300).unwrap();

    let conn = UdpClientConnection::new(zone_config.primary_ns).unwrap();
    let client = SyncClient::with_tsigner(conn, tsigner);

    record.set_data(Some(RData::TXT(TXT::new(Vec::from([cli.proof])))));
    let result = client.create(record, origin).context("Couldn't set record")?;

    Ok(())
}

fn find_record(identifier: &String) -> anyhow::Result<Record> {
    let name = Name::from_str(&("_acme-challenge.".to_owned() + identifier)).context("Invalid input for identifier")?;

    // TODO: traverse CNAMEs, find last CNAME
    // let address = "1.1.1.1:53".parse().unwrap();
    // let conn = UdpClientConnection::new(address).unwrap();
    // let client = SyncClient::new(conn);
    // let response: DnsResponse = client.query(&name, DNSClass::IN, RecordType::TXT).unwrap();
    // let answers: &[Record] = response.answers();
    // println!("answer: {:?}", answers);

    return Ok(Record::with(name,
                           RecordType::TXT,
                           Duration::minutes(1).whole_seconds() as u32));

}
