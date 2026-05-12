use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct RobotsRules {
    pub allow: Vec<String>,
    pub disallow: Vec<String>,
    pub crawl_delay: Option<Duration>,
}

impl RobotsRules {
    /// Checks whether the provided URL path is allowed by robots rules.
    /// Uses longest-prefix match between `Allow` and `Disallow`.
    /// If both have same matched length, `Allow` wins.
    pub fn is_allowed(&self, path: &str) -> bool {
        let mut best_allow = 0usize;
        let mut best_disallow = 0usize;

        for rule in &self.allow {
            if path.starts_with(rule) {
                best_allow = best_allow.max(rule.len());
            }
        }
        for rule in &self.disallow {
            if path.starts_with(rule) {
                best_disallow = best_disallow.max(rule.len());
            }
        }
        if best_allow == 0 && best_disallow == 0 {
            return true;
        }
        match best_allow.cmp(&best_disallow) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => best_allow > 0,
            std::cmp::Ordering::Less => false,
        }
    }

    /// Resolves effective crawl delay:
    /// robots-defined `crawl-delay` has priority, otherwise uses caller fallback.
    pub fn effective_delay(&self, fallback: Duration) -> Duration {
        self.crawl_delay.unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_when_no_rules() {
        let rules = RobotsRules::default();
        assert!(rules.is_allowed("/anything"));
    }
    #[test]
    fn disallow_simple_path() {
        let rules = RobotsRules {
            allow: vec![],
            disallow: vec!["/private".into()],
            crawl_delay: None,
        };
        assert!(!rules.is_allowed("/private/data"));
        assert!(rules.is_allowed("/public"));
    }
    #[test]
    fn allow_overrides_disallow_by_longest_match() {
        let rules = RobotsRules {
            allow: vec!["/public".into()],
            disallow: vec!["/".into()],
            crawl_delay: None,
        };
        assert!(rules.is_allowed("/public/page.html"));
    }
    #[test]
    fn disallow_when_more_specific_than_allow() {
        let rules = RobotsRules {
            allow: vec!["/".into()],
            disallow: vec!["/admin".into()],
            crawl_delay: None,
        };
        assert!(!rules.is_allowed("/admin/panel"));
    }
    #[test]
    fn equal_length_allow_wins() {
        let rules = RobotsRules {
            allow: vec!["/same".into()],
            disallow: vec!["/same".into()],
            crawl_delay: None,
        };
        assert!(rules.is_allowed("/same/page"));
    }
    #[test]
    fn effective_delay_prefers_robots_value() {
        let rules = RobotsRules {
            allow: vec![],
            disallow: vec![],
            crawl_delay: Some(Duration::from_secs(2)),
        };
        assert_eq!(
            rules.effective_delay(Duration::from_millis(100)),
            Duration::from_secs(2)
        );
    }
    #[test]
    fn effective_delay_falls_back_to_config_value() {
        let rules = RobotsRules::default();

        assert_eq!(
            rules.effective_delay(Duration::from_millis(150)),
            Duration::from_millis(150)
        );
    }
}
