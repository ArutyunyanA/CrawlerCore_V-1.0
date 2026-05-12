use crate::robots::policy::RobotsRules;
use tokio::time::{Duration, Instant};

#[derive(Debug)]
pub struct DomainContext {
    pub domain: String,
    pub rules: RobotsRules,
    pub delay: Duration,
    pub last_request: Option<Instant>,
}

impl DomainContext {
    /// Records the timestamp of the latest request to this domain.
    pub fn mark_request(&mut self) {
        self.last_request = Some(Instant::now());
    }

    /// Validates URL path against already parsed robots rules for this domain.
    pub fn allows_path(&self, path: &str) -> bool {
        self.rules.is_allowed(path)
    }

    /// Computes earliest moment when next request is allowed
    /// according to crawl-delay policy.
    pub fn next_allowed_time(&self, now: Instant) -> Instant {
        match self.last_request {
            Some(t) => t + self.delay,
            None => now,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::{Duration, Instant};

    fn ctx_with_delay(delay: Duration) -> DomainContext {
        DomainContext {
            domain: "http://127.0.0.1:3000".into(),
            rules: RobotsRules::default(),
            delay,
            last_request: None,
        }
    }
    #[test]
    fn mark_request_sets_last_request() {
        let mut ctx = ctx_with_delay(Duration::from_secs(1));
        assert!(ctx.last_request.is_none());
        ctx.mark_request();
        assert!(ctx.last_request.is_some());
    }
    #[test]
    fn next_allowed_time_without_previos_request_is_now() {
        let ctx = ctx_with_delay(Duration::from_secs(5));
        let now = Instant::now();
        let allowed = ctx.next_allowed_time(now);
        assert!(allowed >= now);
        assert!(allowed <= now + Duration::from_millis(5));
    }
    #[test]
    fn next_allowed_time_respects_delay() {
        let mut ctx = ctx_with_delay(Duration::from_secs(2));
        ctx.mark_request();
        let t0 = ctx.last_request.unwrap();
        let allowed = ctx.next_allowed_time(Instant::now());
        assert!(allowed >= t0 + Duration::from_secs(2));
    }
}
