// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use std::collections::HashMap;
use std::sync::RwLock;
use trust_dns::op::{Message, MessageType, OpCode, ResponseCode};
use trust_dns::rr::Name;
use trust_dns::rr::dnssec::SupportedAlgorithms;
use trust_dns_server::server::{Request, RequestHandler};
use trust_dns_server::authority::{AuthLookup, Authority};

pub struct DNSHandler {
    authorities: HashMap<Name, RwLock<Authority>>,
}

impl RequestHandler for DNSHandler {
    fn handle_request(&self, request: &Request) -> Message {
        let request_message = &request.message;

        log::trace!("request: {:?}", request_message);

        let response: Message = match request_message.message_type() {
            MessageType::Query => {
                match request_message.op_code() {
                    OpCode::Query => {
                        let response = self.lookup(&request_message);

                        log::trace!("query response: {:?}", response);

                        response
                    }
                    code @ _ => {
                        log::error!("unimplemented opcode: {:?}", code);

                        Message::error_msg(
                            request_message.id(),
                            request_message.op_code(),
                            ResponseCode::NotImp,
                        )
                    }
                }
            }
            MessageType::Response => {
                log::warn!(
                    "got a response as a request from id: {}",
                    request_message.id()
                );

                Message::error_msg(
                    request_message.id(),
                    request_message.op_code(),
                    ResponseCode::NotImp,
                )
            }
        };

        response
    }
}

impl DNSHandler {
    pub fn new() -> Self {
        DNSHandler { authorities: HashMap::new() }
    }

    pub fn upsert(&mut self, name: Name, authority: Authority) {
        self.authorities.insert(name, RwLock::new(authority));
    }

    pub fn lookup(&self, request: &Message) -> Message {
        let mut response: Message = Message::new();

        response.set_id(request.id());
        response.set_op_code(OpCode::Query);
        response.set_message_type(MessageType::Response);
        response.add_queries(request.queries().into_iter().cloned());

        for query in request.queries() {
            if let Some(ref_authority) = self.find_auth_recurse(query.name()) {
                let authority = &ref_authority.read().unwrap();

                log::info!(
                    "request: {} found authority: {}",
                    request.id(),
                    authority.origin()
                );

                let supported_algorithms = SupportedAlgorithms::new();

                let records = authority.search(query, false, supported_algorithms);

                if !records.is_empty() {
                    response.set_response_code(ResponseCode::NoError);
                    response.set_authoritative(true);
                    response.add_answers(records.iter().cloned());

                    let ns_records = authority.ns(false, supported_algorithms);

                    if ns_records.is_empty() {
                        log::warn!("there are no NS records for: {:?}", authority.origin());
                    } else {
                        response.add_name_servers(ns_records.iter().cloned());
                    }
                } else {
                    match records {
                        AuthLookup::NoName => response.set_response_code(ResponseCode::NXDomain),
                        AuthLookup::NameExists => response.set_response_code(ResponseCode::NoError),
                        AuthLookup::Records(..) => {
                            panic!(
                                "programming error, should have return NoError with records above"
                            )
                        }
                    };

                    let soa_records = authority.soa_secure(false, supported_algorithms);

                    if soa_records.is_empty() {
                        log::warn!("there is no SOA record for: {:?}", authority.origin());
                    } else {
                        response.add_name_servers(soa_records.iter().cloned());
                    }
                }
            } else {
                response.set_response_code(ResponseCode::NXDomain);
            }
        }

        response
    }

    fn find_auth_recurse(&self, name: &Name) -> Option<&RwLock<Authority>> {
        let authority = self.authorities.get(name);

        if authority.is_some() {
            return authority;
        } else {
            let name = name.base_name();
            if !name.is_root() {
                return self.find_auth_recurse(&name);
            }
        }

        None
    }
}
