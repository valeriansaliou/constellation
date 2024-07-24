// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

macro_rules! get_cache_store_client {
    ($pools:expr, $error:expr, $client:ident $code:block) => {{
        let mut last_error = $error;

        for pool in $pools {
            let now = Instant::now();

            let pool_delinquent_until = {
                let pool_delinquent_until_read = pool.delinquent_until.read().unwrap();

                pool_delinquent_until_read.unwrap_or(now)
            };

            // Pool not marked as delinquent? Try to acquire
            if pool_delinquent_until <= now {
                // Attempt to get the first healthy pool, in order
                match pool.connection.get().await {
                    Ok(mut $client) => {
                        debug!("acquired cache store client at: {}", pool.target);

                        // Healthy pool acquired, return immediately (break the acquire loop)
                        return $code;
                    }
                    Err(err) => {
                        error!(
                            "could not acquire cache store client from sub-pool: {}",
                            err
                        );

                        // Mark pool as delinquent for some time
                        // Notice: this means that this pool will not be usable for this \
                        //   delinquency time. Also, create a new instant at this moment \
                        //   since we have waited for the connection timeout time, which \
                        //   means that we should not use the current loop pass instant \
                        //   for correctness.
                        *pool.delinquent_until.write().unwrap() = Instant::now()
                            .checked_add(Duration::from_secs(APP_CONF.redis.delinquency_seconds));

                        last_error = $error
                    }
                }
            } else {
                warn!(
                    "skipped acquiring delinquent cache store client from sub-pool: {}",
                    pool.target
                );
            }
        }

        error!("failed getting a cache store client from all pools");

        Err(last_error)
    }};
}
