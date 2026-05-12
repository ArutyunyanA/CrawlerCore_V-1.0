use crate::constants::FIGLET_FONT;
use crate::spiders::link::LinkResult;
use figlet_rs::FIGfont;

pub fn banner_print() {
    let font = FIGfont::from_content(FIGLET_FONT).expect("Failed to load embedded FIGlet font!");
    let banner = font.convert("SHINOBI").unwrap();
    println!("{}", banner);
}

pub fn print_help() {
    println!(
        r#"
Usage:
    crawler [OPTIONS] <url> [url...]

Options:
    --crawling <N>      Crawling concurrency (default: 4)
    --process <N>    Processing concurrency (default: 2)
    --delay <ms>     Delay Between requests (default: 100)
    --robots, -r     Apply robots.txt rules for scheduling
    --help           Show this help
"#
    );
}

pub fn print_link_result(item: &LinkResult) {
    let (status_color, icon) = match item.status {
        200..=299 => ("\x1b[32m", "●"),
        300..=399 => ("\x1b[36m", "↪"),
        400..=499 => ("\x1b[33m", "▲"),
        500..=599 => ("\x1b[31m", "✖"),
        _ => ("x1b[90m", "·"),
    };
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";

    if item.redirected {
        println!(
            "{bold}{status_color}{icon} {status}{reset} {url}\n    {from} {found_on}\n    {to} {final_url}",
            status = item.status,
            url = item.url,
            from = "\x1b[90mRedirected to:\x1b[0m",
            found_on = item.found_on,
            to = "\x1b[90mRedirected to:\x1b[0m",
            final_url = item.final_url.as_deref().unwrap_or("-")
        );
    } else {
        println!(
            "{bold}{status_color}{icon} {status}{reset} {url}\n    {from} {found_on}",
            status = item.status,
            url = item.url,
            from = "\x1b[90mFound on:\x1b[0m",
            found_on = item.found_on
        );
    }
}

pub fn print_fetch_status(url: &str, status: u16) {
    println!("[url] - [code-{}] - {}", status, url);
}

pub fn print_discovered(kind: &str, url: &str) {
    println!("[{}] - {}", kind, url);
}
