use crate::robots::model::RobotsResult;
use reqwest::{Client, StatusCode};
use std::fmt;
use url::Url;

#[derive(Debug)]
enum RobotsUrlError {
    InvalidBase(url::ParseError),
    JoinFailed(url::ParseError),
}

impl fmt::Display for RobotsUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RobotsUrlError::InvalidBase(e) => write!(f, "invalid base url: {}", e),
            RobotsUrlError::JoinFailed(e) => write!(f, "failed to join robots.txt: {}", e),
        }
    }
}

// Join against parsed base URL to correctly preserve scheme/host/port.
fn robots_url(domain: &str) -> Result<String, RobotsUrlError> {
    let base = Url::parse(domain).map_err(RobotsUrlError::InvalidBase)?;
    let url = base
        .join("robots.txt")
        .map_err(RobotsUrlError::JoinFailed)?;
    Ok(url.into())
}

/// Downloads `robots.txt` for a domain and returns status/body snapshot.
///
/// The function is intentionally tolerant:
/// - invalid domain => `400 BAD_REQUEST`,
/// - network failure => `503 SERVICE_UNAVAILABLE`.
pub async fn fetch(client: &Client, domain: &str) -> RobotsResult {
    // Url building
    let url = match robots_url(domain) {
        Ok(u) => u,
        Err(e) => {
            log::debug!("[-] Robots url error for '{}': {}", domain, e);

            return RobotsResult {
                domain: domain.to_string(),
                status: StatusCode::BAD_REQUEST,
                body: None,
            };
        }
    };
    log::info!("[+] Fetching robots.txt: {}", url);

    // Http request
    match client.get(url.clone()).send().await {
        Ok(resp) => {
            let status = resp.status();

            // Body reading
            let body = match resp.text().await {
                Ok(text) => Some(text),
                Err(e) => {
                    log::debug!("[-] Failed read the robots body for '{}': {}", domain, e);
                    None
                }
            };
            RobotsResult {
                domain: domain.to_string(),
                status,
                body,
            }
        }

        // Network error
        Err(e) => {
            log::debug!("[-] Network error fetching roobots for '{}': {}", domain, e);
            RobotsResult {
                domain: domain.to_string(),
                status: StatusCode::SERVICE_UNAVAILABLE,
                body: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::robots_url;

    #[test]
    fn robots_url_keeps_explicit_port() {
        let url = robots_url("http://127.0.0.1:3000").expect("robots url");
        assert_eq!(url, "http://127.0.0.1:3000/robots.txt");
    }

    #[test]
    fn robots_url_for_domain_without_port() {
        let url = robots_url("https://example.com").expect("robots url");
        assert_eq!(url, "https://example.com/robots.txt");
    }
}
