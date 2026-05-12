use std::collections::{HashMap, HashSet, VecDeque};

use tokio::time::Instant;
use url::Url;

use crate::config::domain_key;
use crate::crawler::domain::DomainContext;

pub struct Frontier {
    domains: HashMap<String, DomainContext>,
    queues: HashMap<String, VecDeque<String>>,
    visited: HashSet<String>,
}

impl Frontier {
    /// Creates a frontier with preloaded per-domain robots/crawl contexts.
    pub fn new(domains: HashMap<String, DomainContext>) -> Self {
        Self {
            domains,
            queues: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    /// Inserts URL into domain queue if:
    /// - it has not been visited before,
    /// - domain is known in current crawl session,
    /// - robots policy allows its path.
    pub fn push(&mut self, url: String) {
        if !self.visited.insert(url.clone()) {
            return;
        }
        if let Ok(parsed) = Url::parse(&url) {
            let parsed_domain = match domain_key(&parsed) {
                Ok(domain) => domain,
                Err(_) => return,
            };
            let domain = if self.domains.contains_key(&parsed_domain) {
                parsed_domain
            } else {
                match self.find_compatible_domain(&parsed) {
                    Some(domain) => domain,
                    None => return,
                }
            };
            if let Some(ctx) = self.domains.get(&domain) {
                let path = parsed.path();
                if !ctx.allows_path(path) {
                    return;
                }
            }

            self.queues.entry(domain).or_default().push_back(url);
        }
    }

    fn find_compatible_domain(&self, parsed: &Url) -> Option<String> {
        self.domains.iter().find_map(|(domain, _)| {
            let base = Url::parse(domain).ok()?;
            let base_host = base.host_str()?;
            let parsed_host = parsed.host_str()?;
            let same_host_scope = parsed_host == base_host
                || parsed_host.ends_with(&format!(".{}", base_host))
                || base_host.ends_with(&format!(".{}", parsed_host));

            if same_host_scope
                && base.scheme() == parsed.scheme()
                && base.port_or_known_default() == parsed.port_or_known_default()
            {
                Some(domain.clone())
            } else {
                None
            }
        })
    }

    /// Selects next domain eligible for scheduling at `now`.
    ///
    /// Returns:
    /// - `(domain, allowed_at)` when some queue can be processed now,
    /// - `("", earliest_time)` when all queues are rate-limited,
    /// - `None` when there are no queued URLs.
    pub fn next(&mut self, now: Instant) -> Option<(String, Instant)> {
        let mut selected: Option<Instant> = None;

        for (domain, queue) in &self.queues {
            if queue.is_empty() {
                continue;
            }
            let ctx = match self.domains.get(domain) {
                Some(c) => c,
                None => continue,
            };
            let allowed_at = ctx.next_allowed_time(now);
            if allowed_at <= now {
                return Some((domain.clone(), allowed_at));
            }
            selected = Some(match selected {
                Some(t) => t.min(allowed_at),
                None => allowed_at,
            });
        }
        selected.map(|t| ("".into(), t))
    }

    /// Pops next URL from a specific domain queue and marks request timestamp.
    pub fn take_from(&mut self, domain: &str) -> Option<String> {
        let ctx = self.domains.get_mut(domain)?;
        let queue = self.queues.get_mut(domain)?;
        let url = queue.pop_front()?;
        ctx.mark_request();
        Some(url)
    }
}
