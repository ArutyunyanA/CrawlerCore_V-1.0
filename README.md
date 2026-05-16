# 🕷️ CrawlerCore

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![GitHub last commit](https://img.shields.io/github/last-commit/ArutyunyanA/CrawlerCore_V-1.0?style=flat-square)](https://github.com/ArutyunyanA/CrawlerCore_V-1.0/commits/main)

**High-performance asynchronous web crawler written in Rust** with modular spider architecture, configurable concurrency, and polite crawling defaults.

[Features](#features) • [Quick Start](#quick-start) • [Configuration](#configuration) • [Architecture](#architecture) • [Contributing](#contributing)

</div>

---

## ✨ Features

- **🚀 Asynchronous & Non-blocking**: Built on Tokio for true concurrency with minimal resource overhead
- **🔧 Modular Spider Architecture**: Pluggable spider components for extensibility (HTML parsing, JavaScript, LinkFinder, etc.)
- **⚙️ Configurable Concurrency**: Separate control over crawling workers and processing threads
- **🤖 Robots.txt Compliance**: Built-in support for respecting robots.txt rules
- **⏱️ Request Scheduling**: Configurable delays between requests for polite crawling
- **📊 Multiple Data Extractors**: 
  - HTML href extraction
  - JavaScript file detection
  - LinkFinder for API endpoint discovery
  - HTTP status code logging
- **🎯 Domain Isolation**: Crawls respect domain boundaries with seed URL validation
- **📝 Comprehensive Logging**: env_logger integration with structured output

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Usage](#basic-usage)
  - [CLI Options](#cli-options)
  - [Examples](#examples)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [API Documentation](#api-documentation)
- [Contributing](#contributing)
- [License](#license)

## 🚀 Quick Start

### Installation

**Prerequisites**: Rust 1.70+ and Cargo

```bash
git clone https://github.com/ArutyunyanA/CrawlerCore_V-1.0.git
cd CrawlerCore_V-1.0
cargo build --release
```

Basic Usage

```bash
cargo run --release -- http://example.com
```

📖 Usage
```bash
Usage:
    crawler [OPTIONS] <url> [url...]

Positional Arguments:
    <url> [url...]              Target domain URL(s) to crawl

Options:
    --crawling <N>              Number of concurrent crawling workers (default: 4)
    --process <N>               Number of concurrent processing threads (default: 2)
    --delay <ms>                Delay between requests in milliseconds (default: 100)
    -r, --robots                Respect robots.txt rules during crawling
    -h, --help                  Print help information
```

Examples
Single domain with default settings
```bash
cargo run -- https://example.com
```

Multiple domains with custom concurrency
```bash
cargo run -- \
  --crawling 8 \
  --process 4 \
  https://example.com https://example.org
```

Polite crawling with robots.txt respect and delays
```bash
cargo run -- \
  --crawling 2 \
  --delay 500 \
  --robots \
  https://example.com
```

Custom configuration for penetration testing
```bash
cargo run -- \
  --crawling 16 \
  --process 8 \
  --delay 50 \
  https://target.com
```

Output Format

The crawler outputs discovered resources with metadata:
```bash
[resource_type] - [metadata] - [url]

Examples:
[href]       - http://example.com/page
[javascript] - http://example.com/script.js
[url]        - [code-200] - http://example.com/api/endpoint
[linkfinder] - http://example.com/rest/admin/config
```

⚙️ Configuration

Configuration is managed through command-line arguments and environment variables.
Key Configuration Options
Option	Env Var	Default	Description
--crawling	CRAWLER_CONCURRENCY	4	Number of concurrent HTTP requests
--process	PROCESS_CONCURRENCY	2	Number of URL processing threads
--delay	REQUEST_DELAY_MS	100	Milliseconds between consecutive requests
--robots	RESPECT_ROBOTS_TXT	false	Enforce robots.txt rules
Environment Variables

Set environment variables for persistent configuration:
```bash
export RUST_LOG=info
export CRAWLER_CONCURRENCY=8
export REQUEST_DELAY_MS=200
cargo run -- https://example.com
```

🏗️ Architecture
High-Level Design
```code
┌─────────────────────────────────────────┐
│         Configuration (CLI/Env)         │
└──────────────────┬──────────────────────┘
                   │
┌──────────────────▼──────────────────────┐
│         Crawler Core Engine             │
│  ┌──────────────────────────────────┐  │
│  │   URL Queue (FIFO with dedup)    │  │
│  └────────────┬─────────────────────┘  │
│               │                         │
│  ┌────────────▼──────────────────────┐ │
│  │   Crawling Workers (async/Tokio)  │ │
│  │   - HTTP Requests (reqwest)       │ │
│  │   - Robots.txt Validation         │ │
│  │   - Request Scheduling            │ │
│  └────────────┬──────────────────────┘ │
│               │                         │
│  ┌────────────▼──────────────────────┐ │
│  │   Spider Pipeline (Concurrent)    │ │
│  │   - HTML Parser (scraper)         │ │
│  │   - JS Detector                   │ │
│  │   - LinkFinder (regex analysis)   │ │
│  └────────────┬──────────────────────┘ │
│               │                         │
│  ┌────────────▼──────────────────────┐ │
│  │      Output/Logging                │ │
│  └─────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

Module Structure

    app - Application orchestration and runtime management
    config - Configuration parsing from CLI and environment
    crawler - Core crawling engine with URL queue and worker pool
    spiders - Modular data extraction (href, javascript, linkfinder)
    robots - robots.txt parsing and validation
    user_agent - HTTP User-Agent rotation and header management
    ui - Terminal output and banner rendering
    constants - Application constants and defaults

📦 Dependencies
Crate	Version	Purpose
tokio	1.49	Async runtime with multi-threading support
reqwest	0.13	HTTP client with compression support
scraper	0.25	HTML/CSS parsing
regex	1.12	Pattern matching for link extraction
async-trait	0.1	Async trait definitions
serde	1.0	Serialization framework
futures-util	0.3	Stream and future utilities
log/env_logger	0.4/0.11	Structured logging
🔧 API Documentation
Implementing Custom Spiders

To create a custom spider, implement the Spider trait:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait Spider: Send + Sync {
    async fn process(&self, html: &str, url: &str) -> Vec<String>;
    fn spider_type(&self) -> String;
}

pub struct CustomSpider;

#[async_trait]
impl Spider for CustomSpider {
    async fn process(&self, html: &str, url: &str) -> Vec<String> {
        // Your extraction logic here
        vec![]
    }

    fn spider_type(&self) -> String {
        "custom".to_string()
    }
}
```

Extending the Crawler

The crawler accepts a vector of spiders:
```rust
let spiders: Vec<Box<dyn Spider>> = vec![
    Box::new(HrefSpider),
    Box::new(JavascriptSpider),
    Box::new(LinkFinderSpider),
    Box::new(CustomSpider),
];

crawler.run(spiders).await;
```
📊 Performance Considerations

    Memory: Efficient URL deduplication using HashSet<String>
    CPU: Lock-free concurrent processing with Tokio channels
    I/O: Connection pooling via reqwest with configurable timeouts
    Network: Polite crawling defaults (100ms delay, robots.txt respect)

Tuning for Performance

For maximum throughput (with caution):
```bash
cargo run --release -- \
  --crawling 32 \
  --process 16 \
  --delay 10 \
  https://target.com
```
For minimum resource usage:
```bash
cargo run --release -- \
  --crawling 2 \
  --process 1 \
  --delay 1000 \
  --robots \
  https://target.com
```

🧪 Testing

Run tests with verbose output:
```bash
cargo test -- --nocapture
cargo test --release -- --nocapture
```

Build with all feature flags:
```bash
cargo build --all-features
```
📝 Logging

Control log verbosity via RUST_LOG:
```bash
# Info level (default)
RUST_LOG=info cargo run -- https://example.com

# Debug level (detailed)
RUST_LOG=debug cargo run -- https://example.com

# Trace level (very verbose)
RUST_LOG=trace cargo run -- https://example.com

# Module-specific logging
RUST_LOG=crawler=debug,reqwest=info cargo run -- https://example.com
```
⚖️ Legal & Ethical Use

⚠️ Important: This tool is designed for authorized penetration testing and security research only.

    Always obtain explicit permission before crawling any website
    Respect robots.txt and terms of service
    Use appropriate delays to avoid server overload
    Identify yourself with proper User-Agent headers
    Comply with local laws regarding web crawling and data collection

Misuse of this tool for unauthorized access, data theft, or malicious activities is illegal and unethical.
🤝 Contributing

Contributions are welcome! Please follow these guidelines:

    Fork the repository
    Create a feature branch (git checkout -b feature/amazing-feature)
    Commit your changes (git commit -m 'Add amazing feature')
    Push to the branch (git push origin feature/amazing-feature)
    Open a Pull Request

Code Style

    Follow Rust naming conventions
    Use cargo fmt for formatting
    Run cargo clippy for linting
    Write doc comments for public APIs
```bash
cargo fmt
cargo clippy
cargo test
```
📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
👤 Author

Arutyunyan Artyom - @ArutyunyanA

    📧 Email: arutyunyan_av@icloud.com

🙏 Acknowledgments

    Tokio - Async runtime
    Reqwest - HTTP client
    Scraper - HTML parsing
    Regex - Pattern matching

