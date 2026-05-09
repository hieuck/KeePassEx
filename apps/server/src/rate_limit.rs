//! In-memory rate limiter for auth endpoints
//!
//! Protects against brute-force attacks on login and register endpoints.
//! Uses a sliding window algorithm with per-IP tracking.
//!
//! Limits:
//! - Login: 5 attempts per 15 minutes per IP
//! - Register: 3 attempts per hour per IP

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of attempts in the window
    pub max_attempts: u32,
    /// Window duration
    pub window: Duration,
}

impl RateLimitConfig {
    /// Login: 5 attempts per 15 minutes
    pub fn login() -> Self {
        Self {
            max_attempts: 5,
            window: Duration::from_secs(15 * 60),
        }
    }

    /// Register: 3 attempts per hour
    pub fn register() -> Self {
        Self {
            max_attempts: 3,
            window: Duration::from_secs(60 * 60),
        }
    }
}

/// Per-IP attempt tracking
struct IpRecord {
    attempts: Vec<Instant>,
}

impl IpRecord {
    fn new() -> Self {
        Self {
            attempts: Vec::new(),
        }
    }

    /// Prune attempts outside the window
    fn prune(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.attempts.retain(|&t| t > cutoff);
    }

    /// Count attempts within the window
    fn count(&self) -> u32 {
        self.attempts.len() as u32
    }

    /// Record a new attempt
    fn record(&mut self) {
        self.attempts.push(Instant::now());
    }

    /// Seconds until the oldest attempt expires
    fn retry_after_secs(&self, window: Duration) -> u64 {
        if let Some(&oldest) = self.attempts.first() {
            let expires_at = oldest + window;
            let now = Instant::now();
            if expires_at > now {
                return (expires_at - now).as_secs() + 1;
            }
        }
        0
    }
}

/// Rate limiter — shared across all requests
#[derive(Clone)]
pub struct RateLimiter {
    records: Arc<DashMap<String, IpRecord>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            records: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Check if the IP is rate limited.
    /// Returns `Ok(())` if allowed, `Err(retry_after_secs)` if blocked.
    pub fn check(&self, ip: &str) -> Result<(), u64> {
        let mut record = self
            .records
            .entry(ip.to_string())
            .or_insert_with(IpRecord::new);
        record.prune(self.config.window);

        if record.count() >= self.config.max_attempts {
            let retry_after = record.retry_after_secs(self.config.window);
            return Err(retry_after);
        }

        record.record();
        Ok(())
    }

    /// Reset the rate limit for an IP (e.g., after successful login)
    pub fn reset(&self, ip: &str) {
        self.records.remove(ip);
    }

    /// Periodic cleanup of expired records (call from background task)
    pub fn cleanup(&self) {
        self.records.retain(|_, record| {
            record.prune(self.config.window);
            record.count() > 0
        });
    }
}

/// Global rate limiters for the server
#[derive(Clone)]
pub struct RateLimiters {
    pub login: RateLimiter,
    pub register: RateLimiter,
}

impl RateLimiters {
    pub fn new() -> Self {
        Self {
            login: RateLimiter::new(RateLimitConfig::login()),
            register: RateLimiter::new(RateLimitConfig::register()),
        }
    }
}

impl Default for RateLimiters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_rate_limit_allows_under_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_attempts: 3,
            window: Duration::from_secs(60),
        });

        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
    }

    #[test]
    fn test_login_rate_limit_blocks_over_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_attempts: 3,
            window: Duration::from_secs(60),
        });

        for _ in 0..3 {
            let _ = limiter.check("1.2.3.4");
        }

        // 4th attempt should be blocked
        assert!(limiter.check("1.2.3.4").is_err());
    }

    #[test]
    fn test_different_ips_are_independent() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_attempts: 2,
            window: Duration::from_secs(60),
        });

        let _ = limiter.check("1.1.1.1");
        let _ = limiter.check("1.1.1.1");
        assert!(limiter.check("1.1.1.1").is_err());

        // Different IP should still be allowed
        assert!(limiter.check("2.2.2.2").is_ok());
    }

    #[test]
    fn test_reset_clears_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_attempts: 2,
            window: Duration::from_secs(60),
        });

        let _ = limiter.check("1.2.3.4");
        let _ = limiter.check("1.2.3.4");
        assert!(limiter.check("1.2.3.4").is_err());

        limiter.reset("1.2.3.4");
        assert!(limiter.check("1.2.3.4").is_ok());
    }
}
