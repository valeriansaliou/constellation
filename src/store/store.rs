// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::Duration;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;

use APP_CONF;

pub struct StoreBuilder;

pub struct Store {
    pool: Pool<RedisConnectionManager>,
}

impl StoreBuilder {
    pub fn new() -> Store {
        info!(
            "binding to store backend at {}:{}",
            APP_CONF.redis.host,
            APP_CONF.redis.port
        );

        let addr_auth = match APP_CONF.redis.password {
            Some(ref password) => format!(":{}@", password),
            None => "".to_string(),
        };

        let tcp_addr_raw =
            format!(
            "redis://{}{}:{}/{}",
            &addr_auth,
            APP_CONF.redis.host,
            APP_CONF.redis.port,
            APP_CONF.redis.database,
        );

        debug!("will connect to redis at: {}", tcp_addr_raw);

        match RedisConnectionManager::new(tcp_addr_raw.as_ref()) {
            Ok(manager) => {
                let builder = Pool::builder()
                    .test_on_check_out(false)
                    .max_size(APP_CONF.redis.pool_size)
                    .max_lifetime(Some(
                        Duration::from_secs(APP_CONF.redis.max_lifetime_seconds),
                    ))
                    .idle_timeout(Some(
                        Duration::from_secs(APP_CONF.redis.idle_timeout_seconds),
                    ))
                    .connection_timeout(Duration::from_secs(
                        APP_CONF.redis.connection_timeout_seconds,
                    ));

                match builder.build(manager) {
                    Ok(pool) => {
                        info!("bound to store backend");

                        Store { pool: pool }
                    }
                    Err(_) => panic!("could not spawn redis pool"),
                }
            }
            Err(_) => panic!("could not create redis connection manager"),
        }
    }
}
