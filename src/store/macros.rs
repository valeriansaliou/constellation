// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client {
    ($pool:expr, $error:expr, $client:ident $code:block) => {
        // As DNS server is mono-threaded, it is safe to perform a 'try_get' there, which does \
        //   not wait if no pool is available to serve request answer. This also prevents all \
        //   threads from being blocked in the event of a Redis failure, and thus allow \
        //   Constellation to serve DNS answers from its internal cache.
        match $pool.try_get() {
            Some(mut $client) => $code,
            _ => Err($error),
        }
    };
}
