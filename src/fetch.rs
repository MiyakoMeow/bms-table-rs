//! Data fetching and HTML parsing helpers
//!
//! Provides HTML parsing when the `scraper` feature is enabled, used to extract the header JSON URL from
//! `<meta name="bmstable" content="...">` in a page.
//! Also provides a unified entry to parse a response string into the header JSON or its URL.
//!
//! # Examples
//!
//! ```rust
//! # use bms_table::fetch::{get_web_header_json_value, HeaderQueryContent};
//! let html = r#"
//! <!DOCTYPE html>
//! <html>
//!   <head>
//!     <meta name="bmstable" content="header.json">
//!   </head>
//!   <body></body>
//! </html>
//! "#;
//! match get_web_header_json_value::<serde_json::Value>(html).unwrap() {
//!     HeaderQueryContent::Url(u) => assert_eq!(u, "header.json"),
//!     _ => unreachable!(),
//! }
//! ```
#![cfg(feature = "scraper")]

pub mod reqwest;

use anyhow::{Context, Result, anyhow};
use scraper::{Html, Selector};
use serde::de::DeserializeOwned;

/// Return type of [`get_web_header_json_value`].
///
/// - If the input is HTML, returns the URL extracted from `<meta name="bmstable">`;
/// - If the input is JSON, returns the parsed value of type `T`.
pub enum HeaderQueryContent<T> {
    /// Extracted header JSON URL.
    ///
    /// May be relative or absolute; prefer using `url::Url::join` to resolve.
    Url(String),
    /// Parsed header JSON content.
    Value(T),
}

/// Remove non-printable control characters from JSON text (preserves `\n`, `\r`, `\t`).
///
/// Rationale: some sites return JSON with illegal control characters surrounding it.
/// Cleaning prior to parsing improves compatibility while not affecting preservation of raw text.
#[must_use]
pub fn replace_control_chars(s: &str) -> String {
    s.chars().filter(|ch: &char| !ch.is_control()).collect()
}

/// Parse JSON from a raw string with a cleaning fallback.
///
/// Tries to deserialize from the original `raw` first. If it fails, removes illegal
/// control characters using [`replace_control_chars`] and retries. Returns the parsed
/// value and the raw string that was successfully used.
pub fn parse_json_str_with_fallback<T: DeserializeOwned>(raw: &str) -> Result<(T, String)> {
    match serde_json::from_str::<T>(raw) {
        Ok(v) => Ok((v, raw.to_string())),
        Err(_) => {
            let cleaned = replace_control_chars(raw);
            let v = serde_json::from_str::<T>(&cleaned)?;
            Ok((v, cleaned))
        }
    }
}

/// Parse a response string into the header JSON or its URL.
///
/// Strategy: first attempt to parse as JSON; if it fails, parse as HTML and extract the bmstable URL.
///
/// # Returns
///
/// - `HeaderQueryContent::Json`: input is JSON;
/// - `HeaderQueryContent::Url`: input is HTML.
///
/// # Errors
///
/// Returns an error when the input is HTML but the bmstable field cannot be found.
pub fn get_web_header_json_value<T: DeserializeOwned>(
    response_str: &str,
) -> anyhow::Result<HeaderQueryContent<T>> {
    // First try parsing as JSON (remove illegal control characters before parsing); if it fails, treat as HTML and extract the bmstable URL
    let cleaned = replace_control_chars(response_str);
    match serde_json::from_str::<T>(&cleaned) {
        Ok(header_json) => Ok(HeaderQueryContent::Value(header_json)),
        Err(_) => {
            let bmstable_url = try_extract_bmstable_from_html(response_str)
                .context("When extracting bmstable url")?;
            Ok(HeaderQueryContent::Url(bmstable_url))
        }
    }
}

/// Extract the header query content from a response string with a fallback cleaning step.
///
/// Attempts [`get_web_header_json_value`] on `raw` first; on failure, retries with
/// a control-character-cleaned string via [`replace_control_chars`]. Returns the content
/// and the raw string actually used for the successful extraction.
pub fn header_query_with_fallback<T: DeserializeOwned>(
    raw: &str,
) -> Result<(HeaderQueryContent<T>, String)> {
    match get_web_header_json_value::<T>(raw) {
        Ok(v) => Ok((v, raw.to_string())),
        Err(_) => {
            let cleaned = replace_control_chars(raw);
            let v = get_web_header_json_value::<T>(&cleaned)?;
            Ok((v, cleaned))
        }
    }
}

/// Extract the JSON file URL pointed to by the bmstable field from HTML page content.
///
/// Scans `<meta>` tags looking for elements with `name="bmstable"` and reads their `content` attribute.
///
/// # Errors
///
/// Returns an error when the target tag is not found or `content` is empty.
pub fn try_extract_bmstable_from_html(html_content: &str) -> Result<String> {
    let document = Html::parse_document(html_content);

    // Find all meta tags
    let Ok(meta_selector) = Selector::parse("meta") else {
        return Err(anyhow!("meta tag not found"));
    };

    // 1) Prefer extracting from <meta name="bmstable" content="..."> or <meta property="bmstable">
    for element in document.select(&meta_selector) {
        // Tags whose name or property is bmstable
        let is_bmstable = element
            .value()
            .attr("name")
            .is_some_and(|v| v.eq_ignore_ascii_case("bmstable"))
            || element
                .value()
                .attr("property")
                .is_some_and(|v| v.eq_ignore_ascii_case("bmstable"));
        if is_bmstable
            && let Some(content_attr) = element.value().attr("content")
            && !content_attr.is_empty()
        {
            return Ok(content_attr.to_string());
        }
    }

    // 2) Next, try <link rel="bmstable" href="...json">
    if let Ok(link_selector) = Selector::parse("link") {
        for element in document.select(&link_selector) {
            let rel = element.value().attr("rel");
            let href = element.value().attr("href");
            if rel.is_some_and(|v| v.eq_ignore_ascii_case("bmstable"))
                && let Some(href) = href
                && !href.is_empty()
            {
                return Ok(href.to_string());
            }
        }
    }

    // 3) Then try to find clues for *header*.json in common tag attributes
    //    - a[href], link[href], script[src], meta[content]
    let lower_contains_header_json = |s: &str| {
        let ls = s.to_ascii_lowercase();
        ls.contains("header") && ls.ends_with(".json")
    };

    // a[href]
    if let Ok(a_selector) = Selector::parse("a") {
        for element in document.select(&a_selector) {
            if let Some(href) = element.value().attr("href")
                && lower_contains_header_json(href)
            {
                return Ok(href.to_string());
            }
        }
    }

    // link[href]
    if let Ok(link_selector) = Selector::parse("link") {
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href")
                && lower_contains_header_json(href)
            {
                return Ok(href.to_string());
            }
        }
    }

    // script[src]
    if let Ok(script_selector) = Selector::parse("script") {
        for element in document.select(&script_selector) {
            if let Some(src) = element.value().attr("src")
                && lower_contains_header_json(src)
            {
                return Ok(src.to_string());
            }
        }
    }

    // meta[content]
    for element in document.select(&meta_selector) {
        if let Some(content_attr) = element.value().attr("content")
            && lower_contains_header_json(content_attr)
        {
            return Ok(content_attr.to_string());
        }
    }

    // 4) Finally, a minimal heuristic search on raw text: match substrings containing "header" and ending with .json
    if let Some((start, end)) = find_header_json_in_text(html_content) {
        let candidate = &html_content[start..end];
        return Ok(candidate.to_string());
    }

    Err(anyhow!("bmstable field or header JSON hint not found"))
}

/// Find a substring like "*header*.json" in raw text, returning start/end indices if found.
fn find_header_json_in_text(s: &str) -> Option<(usize, usize)> {
    let lower = s.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(idx) = lower[pos..].find("header") {
        let global_idx = pos + idx;
        // Look for .json after header
        if let Some(json_rel) = lower[global_idx..].find(".json") {
            let end = global_idx + json_rel + ".json".len();
            // Try to find the nearest quote or whitespace before as the start
            let start = lower[..global_idx]
                .rfind(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(global_idx);
            if end > start {
                return Some((start, end));
            }
        }
        pos = global_idx + 6; // skip "header"
    }
    None
}
