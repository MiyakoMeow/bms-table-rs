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

use anyhow::{Context, Result, anyhow};
use reqwest::{
    Client, IntoUrl,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::time::Duration;

use crate::{
    BmsTable, BmsTableData, BmsTableHeader, BmsTableInfo, BmsTableList, BmsTableRaw,
    fetch::{HeaderQueryContent, header_query_with_fallback, parse_json_str_with_fallback},
};

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
    client: &Client,
    web_url: impl IntoUrl,
) -> Result<(BmsTable, BmsTableRaw)> {
    let web_url = web_url.into_url().context("When parsing web url")?;
    let web_response = client
        .get(web_url.clone())
        .send()
        .await
        .context("When fetching web")?
        .text()
        .await
        .context("When parsing web response")?;
    let (hq, web_used_raw) = header_query_with_fallback::<BmsTableHeader>(&web_response)
        .context("When parsing header query")?;
    let (header_url, header, header_raw) = match hq {
        HeaderQueryContent::Url(header_url_string) => {
            let header_url = web_url
                .join(&header_url_string)
                .context("When joining header url")?;
            let header_response = client
                .get(header_url.clone())
                .send()
                .await
                .context("When fetching header")?;
            let header_response_string = header_response
                .text()
                .await
                .context("When parsing header response")?;
            let (hq2, raw2) = header_query_with_fallback::<BmsTableHeader>(&header_response_string)
                .context("When parsing header query")?;
            let HeaderQueryContent::Value(v) = hq2 else {
                return Err(anyhow!(
                    "Cycled header found. web_url: {web_url}, header_url: {header_url_string}"
                ));
            };
            (header_url, v, raw2)
        }
        HeaderQueryContent::Value(value) => (web_url, value, web_used_raw),
    };
    let data_url = header_url
        .join(&header.data_url)
        .context("When joining data url")?;
    let data_response = client
        .get(data_url.clone())
        .send()
        .await
        .context("When fetching web")?
        .text()
        .await
        .context("When parsing web response")?;
    let (data, data_raw_str) = parse_json_str_with_fallback::<BmsTableData>(&data_response)
        .context("When parsing data json")?;
    Ok((
        BmsTable { header, data },
        BmsTableRaw {
            header_json_url: header_url,
            header_raw,
            data_json_url: data_url,
            data_raw: data_raw_str,
        },
    ))
}

/// Fetch and parse a complete BMS difficulty table.
///
/// See [`fetch_table_full`].
pub async fn fetch_table(client: &Client, web_url: impl IntoUrl) -> Result<BmsTable> {
    let (table, _raw) = fetch_table_full(client, web_url)
        .await
        .context("When fetching full table")?;
    Ok(table)
}

/// Fetch a list of BMS difficulty tables.
///
/// Downloads a JSON array from the provided `web_url` and parses it into a list of [`crate::BmsTableInfo`].
/// Each item only requires `name`, `symbol`, and `url` (string); all other fields are collected into `extra`.
pub async fn fetch_table_list(client: &Client, web_url: impl IntoUrl) -> Result<Vec<BmsTableInfo>> {
    let (out, _raw) = fetch_table_list_full(client, web_url)
        .await
        .context("When fetching table list full")?;
    Ok(out)
}

/// Fetch a list of BMS difficulty tables along with the raw JSON string.
///
/// Returns the parsed array of list entries and the raw JSON response text for recording or debugging.
pub async fn fetch_table_list_full(
    client: &Client,
    web_url: impl IntoUrl,
) -> Result<(Vec<BmsTableInfo>, String)> {
    let web_url = web_url.into_url().context("When parsing table list url")?;
    let response_text = client
        .get(web_url)
        .send()
        .await
        .context("When fetching table list")?
        .text()
        .await
        .context("When parsing table list response")?;
    let (list, raw_used) = parse_json_str_with_fallback::<BmsTableList>(&response_text)
        .context("When parsing table list json")?;
    let out: Vec<BmsTableInfo> = list.listes;
    Ok((out, raw_used))
}

/// Create a more lenient and compatible HTTP client.
///
/// - Set a browser-like UA;
/// - Configure timeouts and redirects;
/// - Accept invalid certificates (for a few non-compliant sites);
/// - Accept invalid hostnames (for a few non-compliant sites);
///
/// Note: use `danger_accept_invalid_certs` with caution in production.
pub fn make_lenient_client() -> Result<Client> {
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

    let client = Client::builder()
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
        .context("When building client")?;
    Ok(client)
}
