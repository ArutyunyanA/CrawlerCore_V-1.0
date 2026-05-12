use std::collections::HashMap;

pub const DEFAULT_USER_AGENT: &str = "crawler/1.0";

/// Built in fallback user-agent used when no custom profile is selected.
pub fn builtin_profiles() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("bot", DEFAULT_USER_AGENT),
        (
            "chrome_win",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36",
        ),
        (
            "firefox_linux",
            "Mozilla/5.0 (X11; Linux x86_64; rv:137.0) Gecko/20100101 Firefox/137.0",
        ),
    ])
}

/// Resolves user-agents by a known built-in profile key.
pub fn resolve_profile(profile: &str) -> Result<String, String> {
    let profiles = builtin_profiles();
    profiles
        .get(profile)
        .map(|ua| (*ua).to_string())
        .ok_or_else(|| {
            let mut available = profiles.keys().copied().collect::<Vec<_>>();
            available.sort_unstable();
            format!(
                "Unknown user-agent prfile '{}'. Available profiles: {}",
                profile,
                available.join(", ")
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_profile() {
        let ua = resolve_profile("chrome_win").expect("resolve");
        assert!(us.contains("Mozila/5.0"));
    }

    #[test]
    fn returns_error_for_unknown_profile() {
        let err = resolve_profile("unkown_profile").expect_err("must fail");
        assert!(err.contains("Available profiles"));
    }
}
