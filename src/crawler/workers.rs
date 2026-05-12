use crate::crawler::frontier::Frontier;
use crate::spiders::Spider;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::{Mutex, mpsc};

pub fn spawn_workers<T>(
    concurrency: usize,
    spider: Arc<dyn Spider<Item = T>>,
    job_rx: mpsc::Receiver<String>,
    items_tx: mpsc::Sender<T>,
    frontier: Arc<Mutex<Frontier>>,
    active: Arc<AtomicUsize>,
) where
    T: Send + 'static,
{
    let job_rx = Arc::new(Mutex::new(job_rx));
    for _ in 0..concurrency {
        let spider = spider.clone();
        let items_tx = items_tx.clone();
        let frontier = frontier.clone();
        let active = active.clone();
        let job_rx = job_rx.clone();

        tokio::spawn(async move {
            loop {
                let url = {
                    let mut rx = job_rx.lock().await;
                    rx.recv().await
                };
                let Some(url) = url else { break };
                log::info!("[+] Worker started scrape: {}", url);

                match spider.scrape(url.clone()).await {
                    Ok((items, urls)) => {
                        log::info!(
                            "[+] Worker finished scrape: {} (items={}, discovered={})",
                            url,
                            items.len(),
                            urls.len()
                        );
                        for item in items {
                            let _ = items_tx.send(item).await;
                        }
                        let mut f = frontier.lock().await;
                        for url in urls {
                            f.push(url);
                        }
                    }
                    Err(e) => {
                        log::warn!("scrape error {}: {}", url, e);
                    }
                }
                active.fetch_sub(1, Ordering::SeqCst);
            }
        });
    }
}
