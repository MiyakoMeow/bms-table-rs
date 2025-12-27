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

use std::future::Future;

use anyhow::{Context, Result, anyhow};
use scraper::{ElementRef, Html, Selector};
use serde::de::DeserializeOwned;

use crate::{BmsTable, BmsTableInfo, BmsTableRaw};

/// Result of fetching a table with its raw JSON strings.
pub struct FetchedTable {
    /// Parsed table.
    pub table: BmsTable,
    /// Raw JSON strings and resolved URLs.
    pub raw: BmsTableRaw,
}

/// Result of fetching a table list with its raw JSON string.
pub struct FetchedTableList {
    /// Parsed list entries.
    pub tables: Vec<BmsTableInfo>,
    /// Raw JSON string actually used for parsing.
    pub raw_json: String,
}

/// Unified interface for fetching BMS tables.
pub trait TableFetcher {
    /// Fetch and parse a complete BMS difficulty table, including raw JSON strings.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching or parsing the table fails.
    fn fetch_table(
        &self,
        web_url: url::Url,
    ) -> impl Future<Output = Result<FetchedTable>> + Send + '_;

    /// Fetch a list of BMS difficulty tables, including the raw JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching or parsing the list fails.
    fn fetch_table_list(
        &self,
        web_url: url::Url,
    ) -> impl Future<Output = Result<FetchedTableList>> + Send + '_;
}

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

/// Remove non-printable control characters from JSON text.
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
///
/// # Errors
///
/// Returns an error when both the original and cleaned strings fail to deserialize.
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
/// - `HeaderQueryContent::Value`: input is JSON;
/// - `HeaderQueryContent::Url`: input is HTML.
///
/// # Errors
///
/// Returns an error when the input is HTML but the bmstable field cannot be found.
pub fn get_web_header_json_value<T: DeserializeOwned>(
    response_str: &str,
) -> Result<HeaderQueryContent<T>> {
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
///
/// # Errors
///
/// Returns an error when both attempts fail to extract a header URL or parse JSON.
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
    let meta_selector = Selector::parse("meta").map_err(|_| anyhow!("meta tag not found"))?;
    let link_selector = Selector::parse("link").ok();
    let a_selector = Selector::parse("a").ok();
    let script_selector = Selector::parse("script").ok();

    let find_attr = |selector: &Selector,
                     attr: &str,
                     keep: &mut dyn FnMut(&ElementRef<'_>, &str) -> bool|
     -> Option<String> {
        for element in document.select(selector) {
            if let Some(value) = element.value().attr(attr)
                && keep(&element, value)
            {
                return Some(value.to_string());
            }
        }
        None
    };

    let candidate = meta_bmstable(&document, &meta_selector)
        .or_else(|| {
            let mut keep = |element: &ElementRef<'_>, href: &str| {
                element
                    .value()
                    .attr("rel")
                    .is_some_and(|v| v.eq_ignore_ascii_case("bmstable"))
                    && !href.is_empty()
            };
            link_selector
                .as_ref()
                .and_then(|sel| find_attr(sel, "href", &mut keep))
        })
        .or_else(|| {
            let mut keep = |_: &ElementRef<'_>, href: &str| contains_header_json(href);
            a_selector
                .as_ref()
                .and_then(|sel| find_attr(sel, "href", &mut keep))
        })
        .or_else(|| {
            let mut keep = |_: &ElementRef<'_>, href: &str| contains_header_json(href);
            link_selector
                .as_ref()
                .and_then(|sel| find_attr(sel, "href", &mut keep))
        })
        .or_else(|| {
            let mut keep = |_: &ElementRef<'_>, src: &str| contains_header_json(src);
            script_selector
                .as_ref()
                .and_then(|sel| find_attr(sel, "src", &mut keep))
        })
        .or_else(|| {
            let mut keep = |_: &ElementRef<'_>, content: &str| contains_header_json(content);
            find_attr(&meta_selector, "content", &mut keep)
        })
        .or_else(|| {
            find_header_json_in_text(html_content)
                .map(|(start, end)| html_content[start..end].to_string())
        });

    candidate.map_or_else(
        || Err(anyhow!("bmstable field or header JSON hint not found")),
        Ok,
    )
}

/// Find the start and end indices of a substring like "*header*.json" in raw text.
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

/// Check whether the string contains "header" and ends with ".json".
fn contains_header_json(s: &str) -> bool {
    let ls = s.to_ascii_lowercase();
    ls.contains("header") && ls.ends_with(".json")
}

/// Extract bmstable content from `<meta>` tags.
fn meta_bmstable(document: &Html, meta_selector: &Selector) -> Option<String> {
    for element in document.select(meta_selector) {
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
            return Some(content_attr.to_string());
        }
    }
    None
}
