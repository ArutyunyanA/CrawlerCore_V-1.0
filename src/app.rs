use crate::config::Config;
use crate::crawler::Crawler;
use crate::spiders::js_spider::JsSpider;

use std::sync::Arc;

/// Wires together the default spider implementation and crawler runtime,
/// then starts the asynchronous crawl session.
pub async fn run(config: Config) {
    let spider = Arc::new(JsSpider::new());
    let mut crawler = Crawler::new(config);
    crawler.run(spider).await;
}
