// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hickory_proto::rr::rdata as HickoryRData;
use hickory_proto::rr::record_data::RData;
use hickory_proto::rr::{LowerName, Name, Record, RecordSet, RecordType, RrKey};
use hickory_server::authority::ZoneType;
use hickory_server::server::ServerFuture;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};

use super::handler::{DNSAuthority, DNSHandler};
use crate::APP_CONF;

lazy_static! {
    static ref NAME_SOA_MASTER: Name =
        Name::parse(&APP_CONF.dns.soa_master, Some(&Name::new())).expect("invalid soa master");
    static ref NAME_SOA_RESPONSIBLE: Name =
        Name::parse(&APP_CONF.dns.soa_responsible, Some(&Name::new()))
            .expect("invalid soa responsible");
}

static SERIAL_DEFAULT: u32 = 1;

pub struct DNSListenBuilder;
pub struct DNSListen;

impl DNSListenBuilder {
    pub fn new() -> DNSListen {
        DNSListen {}
    }
}

impl DNSListen {
    #[tokio::main]
    pub async fn run(&self) {
        // Prepare the DNS handler
        // Notice: upsert all available authorities from the configuration.
        let mut handler: DNSHandler = DNSHandler::new();

        for (zone_name, _) in &APP_CONF.dns.zone {
            match Self::zone_authority(&zone_name) {
                Ok((name, authority)) => handler.add_authority(LowerName::new(&name), authority),
                Err(_) => error!("could not load zone {}", zone_name),
            }
        }

        // Run the DNS server
        let mut server = ServerFuture::new(handler);

        // Register sockets & listeners
        for inet in &APP_CONF.dns.inets {
            let udp_socket = UdpSocket::bind(inet)
                .await
                .expect(&format!("udp bind failed: {}", inet));
            let tcp_listener = TcpListener::bind(inet)
                .await
                .expect(&format!("tcp bind failed: {}", inet));

            info!("will listen for udp on {:?}", udp_socket);
            server.register_socket(udp_socket);

            info!("will listen for tcp on {:?}", tcp_listener);
            server.register_listener(tcp_listener, Duration::from_secs(APP_CONF.dns.tcp_timeout));
        }

        // Listen for connections
        info!("listening for dns connections");

        if let Err(err) = server.block_until_done().await {
            error!("failed to listen on dns: {}", err);
        }
    }

    fn zone_authority(zone_name: &str) -> Result<(Name, DNSAuthority), ()> {
        if let Ok(name) = Name::parse(zone_name, Some(&Name::new())) {
            let mut records = BTreeMap::new();

            // Insert base SOA records
            let soa_records = RecordSet::from(Record::from_rdata(
                name.to_owned(),
                APP_CONF.dns.record_ttl,
                RData::SOA(HickoryRData::SOA::new(
                    NAME_SOA_MASTER.to_owned(),
                    NAME_SOA_RESPONSIBLE.to_owned(),
                    SERIAL_DEFAULT,
                    APP_CONF.dns.soa_refresh,
                    APP_CONF.dns.soa_retry,
                    APP_CONF.dns.soa_expire,
                    APP_CONF.dns.soa_ttl,
                )),
            ));

            records.insert(
                RrKey::new(LowerName::new(&name), RecordType::SOA),
                soa_records,
            );

            // Insert base NS records
            let mut ns_records = RecordSet::new(&name, RecordType::NS, SERIAL_DEFAULT);

            for nameserver in &APP_CONF.dns.nameservers {
                ns_records.insert(
                    Record::from_rdata(
                        name.to_owned(),
                        APP_CONF.dns.record_ttl,
                        RData::NS(HickoryRData::NS(
                            Name::parse(nameserver, Some(&Name::new()))
                                .expect("invalid nameserver"),
                        )),
                    ),
                    SERIAL_DEFAULT,
                );
            }

            records.insert(
                RrKey::new(LowerName::new(&name), RecordType::NS),
                ns_records,
            );

            // Build authority instance
            let authority =
                DNSAuthority::new(name.clone(), records, ZoneType::Primary, false).or(Err(()))?;

            Ok((name, authority))
        } else {
            Err(())
        }
    }
}
