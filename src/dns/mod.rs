// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
mod macros;

mod code;
mod handler;

pub mod flatten;
pub mod health;
pub mod listen;
pub mod metrics;
pub mod record;
pub mod zone;
