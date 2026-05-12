/// Result of probing a discovered endpoint or link.
///
/// Carries transport status plus enough context for reporting
/// where the URL was originally found.
#[derive(Debug, Clone)]
pub struct LinkResult {
    pub url: String,
    pub status: u16,
    pub redirected: bool,
    pub final_url: Option<String>,
    pub found_on: String,
}
