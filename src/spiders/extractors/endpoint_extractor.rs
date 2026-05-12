use regex::Regex;
use std::collections::BTreeSet;

pub struct EndpointExtractor {
    endpoint_re: Regex,
}

/// Builds regex-based extractor for likely path-like endpoints
/// embedded inside JavaScript source code.
impl EndpointExtractor {
    pub fn new() -> Self {
        let endpoint_re = Regex::new(
            r#"["'](((?:[a-zA-Z]{1,10}://|//)[^"'/]{1,}\.[a-zA-Z]{2,}[^"']{0,})|((?:/|\../|\./)[^"'><,;| *()(%%$^/\\\[\]][^"'><,;|()]{1,})|([a-zA-Z0-9_\-/]{1,}/[a-zA-Z0-9_\-/]{1,}\.(?:[a-zA-Z]{1,4}|action)(?:[\?|#][^"|']{0,}|))|([a-zA-Z0-9_\-/]{1,}/[a-zA-Z0-9_\-/]{3,}(?:[\?|#][^"|']{0,}|))|([a-zA-Z0-9_\-]{1,}\.(?:php|asp|aspx|jsp|json|action|html|js|txt|xml)(?:[\?|#][^"|']{0,}|)))["']"#,
        ).expect("Invalid endpoint regex");

        Self { endpoint_re }
    }

    /// Extracts unique relative endpoint candidates from JavaScript text.
    ///
    /// The extractor is intentionally conservative and skips malformed
    /// double-slash matches such as `//cdn.example.com`.
    pub fn extract(&self, body: &str) -> Vec<String> {
        let mut endpoints = BTreeSet::new();

        for cap in self.endpoint_re.captures_iter(body) {
            let Some(m) = cap.get(1) else {
                continue;
            };
            let path = m.as_str();
            if path.trim().is_empty() {
                continue;
            }
            endpoints.insert(path.to_string());
        }
        endpoints.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::EndpointExtractor;

    #[test]
    fn extracts_meaningful_paths_from_js_strings() {
        let ex = EndpointExtractor::new();
        let body = r#"const a="/api/v1/users"; const b='text/css'; const c='ftp://127.0.0.1/pub';"#;
        let found = ex.extract(body);
        assert!(found.iter().any(|s| s == "/api/v1/users"));
    }

    #[test]
    fn extract_dot_relative_paths() {
        let ex = EndpointExtractor::new();
        let body = r#"const a='../api/auth/login'; const b='./v2/health'"#;
        let found = ex.extract(body);
        assert!(found.iter().any(|s| s == "../api/auth/login"));
        assert!(found.iter().any(|s| s == "./v2/health"));
    }
}
