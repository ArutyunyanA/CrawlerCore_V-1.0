use crate::robots::policy::RobotsRules;
use std::time::Duration;

fn parse_directive(line: &str) -> Option<(&str, &str)> {
    let (raw_key, raw_value) = line.split_once(":")?;
    let key = raw_key.trim();
    let value = raw_value.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

pub fn parse(txt: &str, user_agent: &str) -> RobotsRules {
    let mut rules = RobotsRules::default();
    let ua = user_agent.to_ascii_lowercase();
    let mut group_matches = false;

    for raw in txt.lines() {
        let line = raw.trim();

        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        let Some((key, value)) = parse_directive(line) else {
            log::debug!("Skipping malformed robots directive: '{}'", line);
            continue;
        };
        let key = key.to_ascii_lowercase();

        match key.as_str() {
            "user-agent" => {
                let agent = value.trim().to_ascii_lowercase();
                group_matches = agent == "*" || (!agent.is_empty() && ua.contains(&agent));
            }
            "allow" if group_matches => {
                rules.allow.push(value.to_string());
            }
            "disallow" if group_matches => {
                rules.disallow.push(value.to_string());
            }
            "crawl-delay" if group_matches => {
                if let Ok(sec) = value.parse::<u64>() {
                    rules.crawl_delay = Some(Duration::from_secs(sec));
                }
            }
            _ => {}
        }
    }
    rules
}
