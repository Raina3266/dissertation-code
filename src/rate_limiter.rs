use std::{num::NonZeroU32, sync::OnceLock};

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

/// A container for global rate limits shared between the whole application
pub struct RateLimiters {
    bbc: DefaultDirectRateLimiter,
    chatgpt: DefaultDirectRateLimiter,
    trends: DefaultDirectRateLimiter,
}

impl RateLimiters {
    /// Get a handle to the global rate limiter object
    pub fn get() -> &'static Self {
        static LIMITERS: OnceLock<RateLimiters> = OnceLock::new();

        LIMITERS.get_or_init(|| {
            // this rate limit is mostly just a ballpark guess. it's fast enough for our purposes, and
            // we never get hung up on with this setting
            let quota = Quota::per_second(NonZeroU32::new(20).unwrap());
            let bbc = RateLimiter::direct(quota);

            // chatgpt rate limit is 90k tokens per minute
            // a token is about 0.75 words (roughly)
            // conservatively, this leaves us 60k words per minute
            // a single description translation task is around 100 words, so 600 translations per
            // minute
            // being extra conservative, let's just call that 500
            let quota = Quota::per_minute(NonZeroU32::new(500).unwrap());
            let chatgpt = RateLimiter::direct(quota);

            // this number comes from the google trends api "quotas" page.
            // It's actually 600, but let's be safe
            let quota = Quota::per_minute(NonZeroU32::new(550).unwrap());
            let trends = RateLimiter::direct(quota);

            Self {
                bbc,
                chatgpt,
                trends,
            }
        })
    }

    pub async fn wait_bbc(&self) {
        self.bbc.until_ready().await;
    }

    pub async fn wait_chatgpt(&self) {
        self.chatgpt.until_ready().await;
    }

    pub async fn wait_trends(&self) {
        self.trends.until_ready().await;
    }
}
