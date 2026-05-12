use scraper::{Html, Selector};
use std::collections::BTreeSet;
use url::Url;

pub struct HtmlParser;

impl HtmlParser {
    /// Creates a stateless HTML parser used to extract crawl candidates
    /// from anchor and script elements.
    pub fn new() -> Self {
        Self
    }

    /// Parses HTML document body and returns absolute URLs discovered on page.
    ///
    /// Currently extracts:
    /// - `<a href=...>` links,
    /// - `<script src=...>` asset references.
    pub fn parse(
        &self,
        page_url: &str,
        body: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut urls = BTreeSet::new();
        let document = Html::parse_document(body);
        let base = Url::parse(page_url)?;

        let link_sel =
            Selector::parse("a[href]").map_err(|e| format!("selector parse error: {:?}", e))?;
        let src_sel =
            Selector::parse("[src]").map_err(|e| format!("selector parse error: {:?}", e))?;
        let form_sel = Selector::parse("form[action]")
            .map_err(|e| format!("selector parse error: {:?}", e))?;

        for elem in document.select(&link_sel) {
            if let Some(href) = elem.value().attr("href") {
                if let Ok(url) = base.join(href) {
                    crate::ui::print_discovered("href", url.as_ref());
                    urls.insert(url.to_string());
                }
            }
        }

        for elem in document.select(&src_sel) {
            if let Some(src) = elem.value().attr("src") {
                if let Ok(url) = base.join(src) {
                    let path = url.path().to_ascii_lowercase();
                    if path.ends_with(".js")
                        || path.ends_with(".json")
                        || path.ends_with(".xml")
                        || path.ends_with(".map")
                    {
                        crate::ui::print_discovered("javascript", url.as_ref());
                    } else {
                        crate::ui::print_discovered("href", url.as_ref());
                    }
                    urls.insert(url.to_string());
                }
            }
        }
        for elem in document.select(&form_sel) {
            if let Some(action) = elem.value().attr("action") {
                if let Ok(url) = base.join(action) {
                    crate::ui::print_discovered("form", url.as_ref());
                    urls.insert(url.to_string());
                }
            }
        }
        Ok(urls.into_iter().collect())
    }
}
