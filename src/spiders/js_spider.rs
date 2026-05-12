use crate::spiders::Spider;
use crate::spiders::link::LinkResult;
use crate::spiders::parsers::html_parser::HtmlParser;
use crate::spiders::parsers::js_parser::JsParser;

use async_trait::async_trait;
use reqwest::Client;

use std::time::Duration;
use tokio::time::timeout;

pub struct JsSpider {
    client: Client,
    html_parser: HtmlParser,
    js_parser: JsParser,
}

impl JsSpider {
    /// Builds spider with HTTP client, HTML parser and JS parser.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().is_empty() {
                    return attempt.follow();
                }
                let prev = attempt.previous().last();
                let next = attempt.url();
                if let Some(prev) = prev {
                    if prev.host_str() == next.host_str() {
                        return attempt.follow();
                    }
                }
                attempt.stop()
            }))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            html_parser: HtmlParser::new(),
            js_parser: JsParser::new(),
        }
    }
}

#[async_trait]
impl Spider for JsSpider {
    type Item = LinkResult;

    /// Human-readable spider identifier used in logs/metrics.
    fn name(&self) -> &str {
        "js-spider"
    }

    /// Fetches URL and delegates extraction based on content type:
    /// - HTML => collect discovered links,
    /// - JavaScript => extract endpoints/link checks.
    async fn scrape(
        &self,
        url: String,
    ) -> Result<(Vec<Self::Item>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.get(&url).send().await?;
        crate::ui::print_fetch_status(&url, resp.status().as_u16());

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = timeout(Duration::from_secs(5), resp.text()).await??;

        if content_type.contains("html") {
            let urls = self.html_parser.parse(&url, &body)?;
            return Ok((Vec::new(), urls));
        }

        if content_type.contains("javascript") || url.ends_with(".js") {
            return self.js_parser.parse(&self.client, &url, &body).await;
        }
        Ok((Vec::new(), Vec::new()))
    }

    /// Handles produced item (currently prints one line per link result).
    async fn process(
        &self,
        item: Self::Item,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        crate::ui::print_link_result(&item);
        Ok(())
    }
}
