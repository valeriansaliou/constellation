// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client {
    ($pools:expr, $error:expr, $client:ident $code:block) => {{
        let mut last_error = $error;

        for (pool, target) in $pools {
            // Attempt to get the first healthy pool, in order
            match pool.get().await {
                Ok(mut $client) => {
                    debug!("acquired cache store client at: {}", target);

                    // Healthy pool acquired, return immediately (break the acquire loop)
                    return $code;
                }
                Err(err) => {
                    warn!(
                        "could not acquire cache store client from sub-pool: {}",
                        err
                    );

                    last_error = $error
                }
            }
        }

        error!("failed getting a cache store client from all pools");

        Err(last_error)
    }};
}
