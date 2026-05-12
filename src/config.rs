use crate::ui::print_help;
use crate::user_agent::{DEFAULT_USER_AGENT, resolve_profile};
use std::env;
use std::time::Duration;
use url::{ParseError, Url};

pub struct Config {
    pub start_targets: Vec<StartTarget>,
    pub crawling_concurrency: usize,
    pub processing_concurrency: usize,
    pub delay_ms: u64,
    pub user_agent: String,
    pub use_robots: bool,
}

#[derive(Debug, Clone)]
pub struct StartTarget {
    pub url: String,
    pub domain: String,
}

#[derive(Debug)]
pub enum DomainBuildError {
    MissingHost(ParseError),
    InvalidPort(ParseError),
}

impl Config {
    /// Parses CLI arguments into crawler runtime configuration.
    ///
    /// This function also:
    /// - handles `--help`,
    /// - applies default concurrency/delay/user-agent values,
    /// - normalizes positional URL arguments into absolute start URLs.
    pub fn from_env() -> Result<Self, String> {
        let mut args = env::args().skip(1).peekable();

        let mut start_targets = Vec::new();
        let mut crawling = 4;
        let mut processing = 2;
        let mut delay_ms = 100;
        let mut user_agent: Option<String> = None;
        let use_robots = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }

                "--user-agent" | "--ua" => {
                    user_agent = Some(take_value::<String>(&mut args, "--user-agent")?);
                }
                "--processing" => {
                    processing = take_value(&mut args, "--process")?;
                }
                "--crawling" => {
                    crawling = take_value(&mut args, "--crawling")?;
                }
                "--delay" => {
                    delay_ms = take_value(&mut args, "--delay")?;
                }
                arg if arg.starts_with("--") => {
                    return Err(format!("Unknown option {}", arg));
                }
                url => {
                    let normalized = normalize_start_url(url);
                    let target = build_start_target(&normalized)
                        .map_err(|e| format!("[-] Invalid start url '{}': {}", e, url))?;
                    start_targets.push(target);
                }
            }
        }
        if start_targets.is_empty() {
            return Err("No start URLs provided...".into());
        }
        let user_agent = resolve_user_agent(user_agent)?;

        Ok(Self {
            start_targets,
            crawling_concurrency: crawling,
            processing_concurrency: processing,
            delay_ms,
            user_agent,
            use_robots,
        })
    }

    /// Converts configured delay from milliseconds into `Duration`
    /// for scheduling and throttling code.
    pub fn default_delay(&self) -> Duration {
        Duration::from_millis(self.delay_ms)
    }
}

/// Converts a CLI URL argument into an absolute URL accepted by `Url::parse`.
///
/// We accept convenience input like `google.com` or `127.0.0.1:3000` in CLI,
/// so this helper infers a scheme only when the user did not provide one.
/// Local loopback hosts default to `http`, public hosts default to `https`.
fn normalize_start_url(raw: &str) -> String {
    if raw.contains("://") {
        return raw.to_string();
    }

    if raw.starts_with("localhost")
        || raw.starts_with("127.0.0.1")
        || raw.starts_with("[::1]")
        || raw.starts_with("::1")
    {
        return format!("http://{}", raw);
    }

    format!("https://{}", raw)
}

/// Builds normalized start target with canonical URL and domain key.
///
/// Domain key format: `scheme://host[:port]`.
fn build_start_target(normalized_url: &str) -> Result<StartTarget, String> {
    let parsed =
        Url::parse(normalized_url).map_err(|e| format!("[-] Failed to parse url: {}", e))?;
    let domain =
        domain_key(&parsed).map_err(|e| format!("[-] Failed to derive domain key: {}", e))?;
    Ok(StartTarget {
        url: parsed.to_string(),
        domain,
    })
}

/// Normalizes URL into a domain key (`scheme://host[:port]`).
pub fn domain_key(url: &Url) -> Result<String, DomainBuildError> {
    let host = match url.host_str() {
        Some(h) if !h.trim().is_empty() => h,
        _ => return Err(DomainBuildError::MissingHost(ParseError::EmptyHost)),
    };
    let mut domain = format!("{}://{}", url.scheme(), host);
    if let Some(port) = url.port() {
        if port == 0 {
            return Err(DomainBuildError::InvalidPort(ParseError::IdnaError));
        }
        domain.push(':');
        domain.push_str(&port.to_string());
    }
    Ok(domain)
}

impl std::fmt::Display for DomainBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainBuildError::MissingHost(parse_error) => {
                write!(f, "Url is missing host ({})", parse_error)
            }
            DomainBuildError::InvalidPort(parse_error) => {
                write!(f, "Url has invalid posrt ({})", parse_error)
            }
        }
    }
}

fn resolve_user_agent(explicit: Option<String>) -> Result<String, String> {
    match explicit {
        Some(profile) => resolve_profile(&profile),
        None => Ok(DEFAULT_USER_AGENT.to_string()),
    }
}

/// Reads and parses the next value after a CLI flag into target type `T`.
///
/// Returns a user-friendly error when the flag is missing a value
/// or when parsing into the requested type fails.
fn take_value<T: std::str::FromStr>(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<T, String> {
    args.next()
        .ok_or_else(|| format!("Missing value for flag {}", flag))?
        .parse()
        .map_err(|_| format!("Invalid value for {}", flag))
}

#[cfg(test)]
mod test {
    use super::{build_start_target, normalize_start_url};

    #[test]
    fn keeps_url_with_scheme() {
        assert_eq!(
            normalize_start_url("http://127.0.0.1:3000"),
            "http://127.0.0.1:3000"
        );
    }

    #[test]
    fn local_hosts_default_to_http() {
        assert_eq!(
            normalize_start_url("127.0.0.1:3000"),
            "http://127.0.0.1:3000"
        );
        assert_eq!(
            normalize_start_url("localhost:3000"),
            "http://localhost:3000"
        );
    }

    #[test]
    fn public_hosts_default_to_https() {
        assert_eq!(normalize_start_url("google.com"), "https://google.com");
    }

    #[test]
    fn ipv6_loopback_default_to_http() {
        assert_eq!(normalize_start_url("[::1]:3000"), "http://[::1]:3000");
        assert_eq!(normalize_start_url("::1"), "http://::1");
    }

    #[test]
    fn builds_domain_with_explicit_port() -> Result<(), String> {
        let target = build_start_target("http://127.0.0.1:3000")?;
        assert_eq!(target.domain, "http://127.0.0.1:3000");
        Ok(())
    }

    #[test]
    fn builds_domain_without_port() -> Result<(), String> {
        let target = build_start_target("https://google.com")?;
        assert_eq!(target.domain, "https://google.com");
        Ok(())
    }
}
