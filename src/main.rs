mod app;
mod config;
mod constants;
mod crawler;
mod robots;
mod spiders;
mod ui;
mod user_agent;

use crate::ui::banner_print;

/// Application entrypoint:
/// initializes logging/UI, loads runtime configuration,
/// and starts the crawl pipeline.
#[tokio::main]
async fn main() {
    env_logger::init();
    banner_print();
    let config = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            ui::print_help();
            std::process::exit(1);
        }
    };
    app::run(config).await;
}
