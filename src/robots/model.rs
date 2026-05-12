use reqwest::StatusCode;

#[derive(Debug)]
pub struct RobotsResult {
    pub domain: String,
    pub status: StatusCode,
    pub body: Option<String>,
}
