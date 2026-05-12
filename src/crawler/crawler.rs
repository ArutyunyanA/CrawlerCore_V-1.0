use crate::config::Config;
use crate::crawler::domain::DomainContext;
use crate::crawler::frontier::Frontier;
use crate::crawler::workers::spawn_workers;
use crate::robots::fetcher::fetch;
use crate::robots::parse::parse;
use crate::robots::policy::RobotsRules;
use crate::spiders::Spider;

use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, Instant};

pub struct Crawler {
    config: Config,
}

/// Creates crawler runtime with CLI/environment configuration.
impl Crawler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs full crawl pipeline:
    /// 1) load robots policies for seed domains,
    /// 2) initialize frontier and worker pool,
    /// 3) schedule jobs with per-domain crawl-delay,
    /// 4) process discovered spider items.
    pub async fn run<T>(&mut self, spider: Arc<dyn Spider<Item = T>>)
    where
        T: Send + 'static,
    {
        let robots_client = reqwest::Client::builder()
            .user_agent(self.config.user_agent.clone())
            .build()
            .expect("failed to build robots client");
        let mut domains: HashMap<String, DomainContext> = HashMap::new();
        let mut start_urls: Vec<String> = Vec::new();
        let fallback_delay = Duration::from_millis(self.config.delay_ms);

        for target in &self.config.start_targets {
            let robots_result = fetch(&robots_client, &target.domain).await;
            log::info!(
                "[✓] robots fetched for {} with status {}",
                robots_result.domain,
                robots_result.status,
            );

            let parsed_rules = if robots_result.status.is_success() {
                robots_result
                    .body
                    .as_deref()
                    .map(|txt| parse(txt, &self.config.user_agent))
                    .unwrap_or_else(RobotsRules::default)
            } else {
                RobotsRules::default()
            };

            for path in parsed_rules
                .allow
                .iter()
                .chain(parsed_rules.disallow.iter())
            {
                if !path.starts_with('/') {
                    continue;
                }
                let discovered = format!("{}{}", target.domain, path);
                crate::ui::print_discovered("robots", &discovered);
                start_urls.push(discovered);
            }

            let rules = if self.config.use_robots {
                parsed_rules
            } else {
                RobotsRules::default()
            };

            let delay = rules.effective_delay(fallback_delay);
            let domain_key = robots_result.domain;

            domains
                .entry(domain_key.clone())
                .or_insert_with(|| DomainContext {
                    domain: domain_key,
                    rules,
                    delay,
                    last_request: None,
                });

            start_urls.push(target.url.clone());
        }
        if start_urls.is_empty() {
            log::warn!("no valid start urls provided - crawler will not start");
            return;
        }
        println!("[+] Target domains loaded: {}", domains.len());
        println!("[+] Seed URLs queued: {}", start_urls.len());
        let mut frontier = Frontier::new(domains);
        for url in start_urls {
            frontier.push(url)
        }
        let frontier = Arc::new(Mutex::new(frontier));
        let active = Arc::new(AtomicUsize::new(0));

        let (job_tx, job_rx) = mpsc::channel::<String>(100);
        let (items_tx, mut items_rx) = mpsc::channel::<T>(100);

        spawn_workers(
            self.config.crawling_concurrency,
            spider.clone(),
            job_rx,
            items_tx.clone(),
            frontier.clone(),
            active.clone(),
        );

        let scheduler = {
            let frontier = frontier.clone();
            let job_tx = job_tx.clone();
            let active = active.clone();
            tokio::spawn(async move {
                loop {
                    let decision = {
                        let mut f = frontier.lock().await;
                        f.next(Instant::now())
                    };
                    match decision {
                        Some((domain, when)) if domain.is_empty() => {
                            tokio::time::sleep_until(when).await;
                        }
                        Some((domain, _)) => {
                            let next_url = {
                                let mut f = frontier.lock().await;
                                f.take_from(&domain)
                            };
                            if let Some(url) = next_url {
                                active.fetch_add(1, Ordering::SeqCst);
                                if job_tx.send(url).await.is_err() {
                                    break;
                                }
                            }
                        }
                        None => {
                            if active.load(Ordering::SeqCst) == 0 {
                                break;
                            }
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                }
            })
        };
        let processor = {
            let spider = spider.clone();
            tokio::spawn(async move {
                let mut processed = 0usize;
                while let Some(item) = items_rx.recv().await {
                    let _ = spider.process(item).await;
                    processed += 1;
                }
                processed
            })
        };
        drop(job_tx);
        let _ = scheduler.await;

        drop(items_tx);
        let processed = processor.await.unwrap_or(0);
        println!("[✓] Crawl finished. Processed link checks: {}", processed);
    }
}
