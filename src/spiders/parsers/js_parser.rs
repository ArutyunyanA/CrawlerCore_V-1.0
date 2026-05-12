use crate::spiders::extractors::endpoint_extractor::EndpointExtractor;
use crate::spiders::link::LinkResult;

use futures_util::stream::{self, StreamExt};
use reqwest::Client;
use std::collections::BTreeSet;
use std::error::Error;
use url::Url;

pub struct JsParser {
    extractor: EndpointExtractor,
}

impl JsParser {
    /// Creates JavaScript parser with endpoint extractor dependency.
    pub fn new() -> Self {
        Self {
            extractor: EndpointExtractor::new(),
        }
    }

    /// Parses JavaScript body, resolves extracted endpoints against `js_url`,
    /// probes them over HTTP, and returns both probe results and URLs
    /// discovered for further crawl scheduling.
    pub async fn parse(
        &self,
        client: &Client,
        js_url: &str,
        body: &str,
    ) -> Result<(Vec<LinkResult>, Vec<String>), Box<dyn Error + Send + Sync>> {
        let mut results = Vec::new();
        let mut discovered = BTreeSet::new();
        let base = Url::parse(js_url)?;
        let endpoints = self.extractor.extract(body);

        let mut probe_targets = BTreeSet::new();
        for endpoint in endpoints {
            for abs in build_candidate_urls(&base, &endpoint) {
                crate::ui::print_discovered("linkfinder", &abs);
                discovered.insert(abs.clone());
                probe_targets.insert(abs);
            }
        }
        let found_on = js_url.to_string();
        let client_ref = client;
        let mut responses = stream::iter(probe_targets.into_iter().map(|abs| {
            let client = client_ref;
            let found_on = found_on.clone();
            async move {
                let resp = client.get(&abs).send().await.ok()?;
                Some(LinkResult {
                    url: abs,
                    status: resp.status().as_u16(),
                    redirected: false,
                    final_url: None,
                    found_on,
                })
            }
        }))
        .buffer_unordered(20);

        while let Some(item) = responses.next().await {
            if let Some(link) = item {
                results.push(link);
            }
        }
        Ok((results, discovered.into_iter().collect()))
    }
}

fn build_candidate_urls(base: &Url, endpoint: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(u) = base.join(endpoint) {
        out.push(u.to_string());
    }
    if let Some(host) = base.host_str() {
        let mut root = format!("{}://{}", base.scheme(), host);
        if let Some(port) = base.port() {
            root.push(':');
            root.push_str(&port.to_string());
        }
        root.push('/');
        if let Ok(root_url) = Url::parse(&root) {
            if let Ok(u) = root_url.join(endpoint) {
                out.push(u.to_string());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn is_probable_url_candidate(candidate: &str) -> bool {
    let c = candidate.trim().to_ascii_lowercase();
    if c.is_empty() {
        return false;
    }

    // Frequently false-positive MIME/content-type tokens surfaced by LinkFinder regex.
    if c.contains("/")
        && (c.starts_with("text/")
            || c.starts_with("application/")
            || c.starts_with("image/")
            || c.starts_with("audio/")
            || c.starts_with("video/"))
    {
        return false;
    }

    // Skip bare runtime/module identifiers that are not fetchable paths.
    if !c.starts_with('/')
        && !c.starts_with("./")
        && !c.starts_with("../")
        && !c.starts_with("http://")
        && !c.starts_with("https://")
        && !c.starts_with("//")
        && !c.contains('.')
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_probable_url_candidate;

    #[test]
    fn filters_content_type_false_positives() {
        assert!(!is_probable_url_candidate(
            "application/x-www-form-urlencoded"
        ));
        assert!(!is_probable_url_candidate("text/html"));
        assert!(!is_probable_url_candidate("image/x-icon"));
    }

    #[test]
    fn keeps_real_relative_and_absolute_urls() {
        assert!(is_probable_url_candidate("/api/v1/users"));
        assert!(is_probable_url_candidate("chunk-24EZLZ4I.js"));
        assert!(is_probable_url_candidate("https://example.com/app.js"));
    }
}
