// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use std::time::Duration;
use std::net::{TcpListener, UdpSocket};
use trust_dns_server::server::ServerFuture;
use trust_dns_server::authority::Catalog;

use APP_CONF;

pub struct DNSListenBuilder;
pub struct DNSListen;

impl DNSListenBuilder {
    pub fn new() -> DNSListen {
        DNSListen {}
    }
}

impl DNSListen {
    pub fn run(&self) {
        // Run the DNS server
        let catalog: Catalog = Catalog::new();

        // TODO: register entries, or make catalog global somewhat?

        let mut server = ServerFuture::new(catalog).expect("error creating dns server");

        // Register sockets & listeners
        for inet in &APP_CONF.dns.inets {
            let udp_socket = UdpSocket::bind(inet).expect(&format!("udp bind failed: {}", inet));
            let tcp_listener =
                TcpListener::bind(inet).expect(&format!("tcp bind failed: {}", inet));

            log::info!("listening for udp on {:?}", udp_socket);
            server.register_socket(udp_socket);

            log::info!("listening for tcp on {:?}", tcp_listener);
            server
                .register_listener(tcp_listener, Duration::from_secs(APP_CONF.dns.tcp_timeout))
                .expect("could not register tcp listener");
        }

        // Listen for connections
        log::info!("listening for dns connections");

        if let Err(err) = server.listen() {
            log::error!("failed to listen on dns: {}", err);
        }
    }
}
