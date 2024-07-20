// Constellation
//
// Pluggable authoritative DNS server
// Copyright: 2019, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::thread;
use std::time::{Duration, Instant};

use super::cache::StoreCacheFlush;

pub struct StoreFlushBuilder;
pub struct StoreFlush;

const FLUSH_PERFORM_INTERVAL: Duration = Duration::from_secs(20);

impl StoreFlushBuilder {
    pub fn new() -> StoreFlush {
        StoreFlush {}
    }
}

impl StoreFlush {
    #[tokio::main]
    pub async fn run(&self) {
        info!("store flusher is now active");

        loop {
            // Hold for next aggregate run
            thread::sleep(FLUSH_PERFORM_INTERVAL);

            debug!("running a store flush...");

            let flush_start = Instant::now();

            Self::perform().await;

            let flush_took = flush_start.elapsed();

            info!(
                "ran store flush (took {}s + {}ms)",
                flush_took.as_secs(),
                flush_took.subsec_millis()
            );
        }
    }

    async fn perform() {
        // Proceed all perform actions

        // #1: Flush expired cache
        StoreCacheFlush::expire();

        // #2: Flush to-be-refreshed cache
        StoreCacheFlush::refresh().await;
    }
}
