use crate::config::{Config, CrawlMode};
use crate::crawler::Crawler;
use crate::spiders::Spider;
use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub struct MockSpider {
    pub calls: Arc<AtomicUsize>,
}

impl MockSpider {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl Spider for MockSpider {
    type Item = ();

    async fn scrape(
        &self,
        _url: String,
    ) -> anyhow::Result<(Vec<()>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok((vec![], vec![]))
    }

    async fn process(
        &self,
        item: (),
    ) -> anyhow::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn name(&self) -> &str {
        "mock"
    }
}

use httpmock::prelude::*;

fn mock_server_with_robots(robots_body: &str) -> MockServer {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(GET).path("/robots.txt");
        then.status(200)
            .header("Content-Type", "text/plain")
            .body(robots_body);
    });
    server.mock(|when, then| {
        when.method(GET).path("/index.html");
        then.status(200)
            .header("Content-Type", "text/html")
            .body("<html><body>ok</body></html>");
    });
    server
}

#[tokio::test]
async fn crawler_respects_robots_disallow() {
    let server = mock_server_with_robots(
        r#"
            User-agent: *
            Disallow: /
        "#,
    );

    let start_url = format!("{}/index.html", server.base_url());
    let config = Config {
        start_urls: vec![start_url],
        crawling_concurrency: 1,
        processing_concurrency: 1,
        delay_ms: 0,
        mode: CrawlMode::StaticHtml,
        user_agent: "test-bot".into(),
    };
    let spider = Arc::new(MockSpider::new());
    let calls = spider.calls.clone();
    let mut crawler = Crawler::new(config);
    crawler.run(spider).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn crawler_allows_when_permitted() {
    let server = mock_server_with_robots(
        r#"
            User-agent: *
            Allow: /
        "#,
    );
    let start_url = format!("{}/index.html", server.base_url());
    let config = Config {
        start_urls: vec![start_url],
        crawling_concurrency: 1,
        processing_concurrency: 1,
        delay_ms: 0,
        mode: CrawlMode::StaticHtml,
        user_agent: "test-bot".into(),
    };

    let spider = Arc::new(MockSpider::new());
    let calls = spider.calls.clone();
    let mut crawler = Crawler::new(config);
    crawler.run(spider).await;
    assert!(calls.load(Ordering::SeqCst) >= 1);
}
