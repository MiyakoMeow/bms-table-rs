//! Network fetching module based on `reqwest`
//!
//! Provides an all-in-one ability to fetch and parse BMS difficulty tables from a web page or a header JSON source:
//! - Fetch the page and extract the bmstable header URL from HTML (if present);
//! - Download and parse the header JSON;
//! - Download and parse chart data according to `data_url` in the header;
//! - Return a `BmsTable` containing the header and the chart set.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::{fetch_table, make_lenient_client};
//! let client = make_lenient_client()?;
//! let table = fetch_table(&client, "https://stellabms.xyz/sl/table.html").await?;
//! assert!(!table.data.charts.is_empty());
//! # Ok(())
//! # }
//! ```
#![cfg(feature = "reqwest")]
use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::time::Duration;
use url::Url;

use crate::{BmsTable, BmsTableInfo, BmsTableRaw, fetch::replace_control_chars};

/// Fetch and parse a complete BMS difficulty table from a web page or a header JSON source.
///
/// # Parameters
///
/// - `web_url`: page URL or an URL pointing directly to the header JSON.
///
/// # Returns
///
/// Parsed [`crate::BmsTable`], containing header and chart data.
///
/// # Errors
///
/// - Network request failures (connection failure, timeout, etc.)
/// - Response content cannot be parsed as HTML/JSON or structure is unexpected
/// - Header JSON does not contain `data_url` or has the wrong type
pub async fn fetch_table_full(
    client: &reqwest::Client,
    web_url: &str,
) -> Result<(BmsTable, BmsTableRaw)> {
    let web_url = Url::parse(web_url)?;
    let web_response = client
        .get(web_url.clone())
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let (header_url, header_json, header_raw) =
        match crate::fetch::get_web_header_json_value(&web_response)? {
            crate::fetch::HeaderQueryContent::Url(header_url_string) => {
                let header_url = web_url.join(&header_url_string)?;
                let header_response = client
                    .get(header_url.clone())
                    .send()
                    .await
                    .map_err(|e| anyhow!("When fetching header: {e}"))?;
                let header_response_string = header_response
                    .text()
                    .await
                    .map_err(|e| anyhow!("When parsing header response: {e}"))?;
                let crate::fetch::HeaderQueryContent::Json(header_json) =
                    crate::fetch::get_web_header_json_value(&header_response_string)?
                else {
                    return Err(anyhow!(
                        "Cycled header found. web_url: {web_url}, header_url: {header_url_string}"
                    ));
                };
                (header_url, header_json, header_response_string)
            }
            crate::fetch::HeaderQueryContent::Json(value) => {
                let header_raw = serde_json::to_string(&value)?;
                (web_url, value, header_raw)
            }
        };
    let data_url_str = header_json
        .get("data_url")
        .ok_or_else(|| anyhow!("\"data_url\" not found in header json!"))?
        .as_str()
        .ok_or_else(|| anyhow!("\"data_url\" is not a string!"))?;
    let data_url = header_url.join(data_url_str)?;
    let data_response = client
        .get(data_url.clone())
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    // Remove illegal control characters before parsing while keeping the original data_raw unchanged
    let data_cleaned = replace_control_chars(&data_response);
    let data_json: Value = serde_json::from_str(&data_cleaned)?;
    // Build BmsTable via the crate's deserialization
    let header: crate::BmsTableHeader = serde_json::from_value(header_json)
        .map_err(|e| anyhow!("When parsing header json: {e}"))?;
    let data: crate::BmsTableData =
        serde_json::from_value(data_json).map_err(|e| anyhow!("When parsing data json: {e}"))?;
    Ok((
        BmsTable { header, data },
        BmsTableRaw {
            header_json_url: header_url,
            header_raw,
            data_json_url: data_url,
            data_raw: data_response,
        },
    ))
}

/// Fetch and parse a complete BMS difficulty table.
///
/// See [`fetch_table_full`].
pub async fn fetch_table(client: &reqwest::Client, web_url: &str) -> Result<BmsTable> {
    let (table, _raw) = fetch_table_full(client, web_url).await?;
    Ok(table)
}

/// Fetch a list of BMS difficulty tables.
///
/// Downloads a JSON array from the provided `web_url` and parses it into a list of [`crate::BmsTableInfo`].
/// Each item only requires `name`, `symbol`, and `url` (string); all other fields are collected into `extra`.
pub async fn fetch_table_list(
    client: &reqwest::Client,
    web_url: &str,
) -> Result<Vec<BmsTableInfo>> {
    let (out, _raw) = fetch_table_list_full(client, web_url).await?;
    Ok(out)
}

/// Fetch a list of BMS difficulty tables along with the raw JSON string.
///
/// Returns the parsed array of list entries and the raw JSON response text for recording or debugging.
pub async fn fetch_table_list_full(
    client: &reqwest::Client,
    web_url: &str,
) -> Result<(Vec<BmsTableInfo>, String)> {
    let web_url = Url::parse(web_url)?;
    let response_text = client
        .get(web_url)
        .send()
        .await
        .map_err(|e| anyhow!("When fetching table list: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing table list response: {e}"))?;

    // Remove illegal control characters before parsing while keeping the original response text unchanged
    let cleaned = replace_control_chars(&response_text);
    let value: Value = serde_json::from_str(&cleaned)?;
    let arr = value
        .as_array()
        .ok_or_else(|| anyhow!("Table list root is not an array"))?;

    let mut out = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        let obj = item
            .as_object()
            .ok_or_else(|| anyhow!("Table list item #{idx} is not an object"))?;

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field 'name' at index {idx}"))?;
        let symbol = obj
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field 'symbol' at index {idx}"))?;
        let url_str = obj
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field 'url' at index {idx}"))?;
        let url = Url::parse(url_str)?;

        #[cfg(feature = "serde")]
        let extra = {
            let mut m: BTreeMap<String, Value> = BTreeMap::new();
            for (k, v) in obj.iter() {
                if k != "name" && k != "symbol" && k != "url" {
                    m.insert(k.clone(), v.clone());
                }
            }
            m
        };

        let entry = BmsTableInfo {
            name: name.to_string(),
            symbol: symbol.to_string(),
            url,
            #[cfg(feature = "serde")]
            extra,
        };
        out.push(entry);
    }

    Ok((out, response_text))
}

/// Create a more lenient and compatible HTTP client.
///
/// - Set a browser-like UA;
/// - Configure timeouts and redirects;
/// - Accept invalid certificates (for a few non-compliant sites);
/// - Accept invalid hostnames (for a few non-compliant sites);
///
/// Note: use `danger_accept_invalid_certs` with caution in production.
pub fn make_lenient_client() -> Result<reqwest::Client> {
    // Default headers emulate real browser behavior more closely
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("accept"),
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9",
        ),
    );
    headers.insert(
        HeaderName::from_static("accept-language"),
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static("1"),
    );
    headers.insert(
        HeaderName::from_static("connection"),
        HeaderValue::from_static("keep-alive"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119 Safari/537.36 bms-table-rs")
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::limited(100))
        // Automatically include Referer on redirects, closer to browser behavior
        .referer(true)
        // Enable cookie store, closer to real user sessions
        .cookie_store(true)
        // Keep lenient TLS settings for compatibility with some non-compliant sites
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| anyhow!("When building client: {e}"))?;
    Ok(client)
}
