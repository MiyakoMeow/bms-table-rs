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
//! use bms_table::fetch::reqwest::Fetcher;
//! let fetcher = Fetcher::lenient()?;
//! let table = fetcher.fetch_table("https://stellabms.xyz/sl/table.html").await?;
//! assert!(!table.data.charts.is_empty());
//! # Ok(())
//! # }
//! ```
#![cfg(feature = "reqwest")]

use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use reqwest::{
    Client, IntoUrl,
    header::{HeaderMap, HeaderName, HeaderValue},
};

use crate::{
    BmsTable, BmsTableData, BmsTableHeader, BmsTableInfo, BmsTableList, BmsTableRaw,
    fetch::{HeaderQueryContent, header_query_with_fallback, parse_json_str_with_fallback},
};

/// Fetcher wrapper around a reusable [`reqwest::Client`].
///
/// Provides an ergonomic, one-stop API for fetching a table (or table list) from a web URL.
#[derive(Clone)]
pub struct Fetcher {
    /// Underlying HTTP client.
    client: Client,
}

impl Fetcher {
    /// Create a fetcher from an existing [`reqwest::Client`].
    #[must_use]
    pub const fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a fetcher with a more compatible, browser-like HTTP client configuration.
    ///
    /// See [`make_lenient_client`] for the exact settings.
    ///
    /// # Errors
    ///
    /// Returns an error if building the underlying HTTP client fails.
    pub fn lenient() -> Result<Self> {
        Ok(Self::new(make_lenient_client()?))
    }

    /// Borrow the underlying [`reqwest::Client`].
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.client
    }

    /// Fetch and parse a complete BMS difficulty table.
    ///
    /// # Errors
    ///
    /// Propagates network, parsing, and join errors from [`fetch_table`].
    pub async fn fetch_table(&self, web_url: impl IntoUrl) -> Result<BmsTable> {
        fetch_table(&self.client, web_url).await
    }

    /// Fetch and parse a complete BMS difficulty table, including raw JSON strings.
    ///
    /// # Errors
    ///
    /// Propagates network, parsing, and join errors from [`fetch_table_full`].
    pub async fn fetch_table_with_raw(&self, web_url: impl IntoUrl) -> Result<FetchTableOutput> {
        let (table, raw) = fetch_table_full(&self.client, web_url).await?;
        Ok(FetchTableOutput { table, raw })
    }

    /// Fetch a list of BMS difficulty tables.
    ///
    /// # Errors
    ///
    /// Propagates network and parsing errors from [`fetch_table_list`].
    pub async fn fetch_table_list(&self, web_url: impl IntoUrl) -> Result<Vec<BmsTableInfo>> {
        fetch_table_list(&self.client, web_url).await
    }

    /// Fetch a list of BMS difficulty tables, including the raw JSON string.
    ///
    /// # Errors
    ///
    /// Propagates network and parsing errors from [`fetch_table_list_full`].
    pub async fn fetch_table_list_with_raw(
        &self,
        web_url: impl IntoUrl,
    ) -> Result<FetchTableListOutput> {
        let (tables, raw_json) = fetch_table_list_full(&self.client, web_url).await?;
        Ok(FetchTableListOutput { tables, raw_json })
    }
}

/// Result of fetching a table with its raw JSON strings.
pub struct FetchTableOutput {
    /// Parsed table.
    pub table: BmsTable,
    /// Raw JSON strings and resolved URLs.
    pub raw: BmsTableRaw,
}

/// Result of fetching a table list with its raw JSON string.
pub struct FetchTableListOutput {
    /// Parsed list entries.
    pub tables: Vec<BmsTableInfo>,
    /// Raw JSON string actually used for parsing.
    pub raw_json: String,
}

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
    let web_url = web_url.into_url().context("When parsing target url")?;

    let web_page_text = client
        .get(web_url.clone())
        .send()
        .await
        .context("When fetching web page")?
        .text()
        .await
        .context("When reading web page body")?;

    let (web_header_query, web_used_text) =
        header_query_with_fallback::<BmsTableHeader>(&web_page_text)
            .context("When extracting header query from web page")?;

    let (header_json_url, header, header_raw) = match web_header_query {
        HeaderQueryContent::Url(header_url_string) => {
            let header_json_url = web_url
                .join(&header_url_string)
                .context("When resolving header json url")?;

            let header_text = client
                .get(header_json_url.clone())
                .send()
                .await
                .context("When fetching header json")?
                .text()
                .await
                .context("When reading header json body")?;

            let (header_query2, header_used_text) =
                header_query_with_fallback::<BmsTableHeader>(&header_text)
                    .context("When parsing header json")?;

            let HeaderQueryContent::Value(header) = header_query2 else {
                return Err(anyhow!(
                    "Cycled header found. web_url: {web_url}, header_url: {header_url_string}"
                ));
            };

            (header_json_url, header, header_used_text)
        }
        HeaderQueryContent::Value(header) => (web_url.clone(), header, web_used_text),
    };

    let data_json_url = header_json_url
        .join(&header.data_url)
        .context("When resolving data json url")?;

    let data_text = client
        .get(data_json_url.clone())
        .send()
        .await
        .context("When fetching data json")?
        .text()
        .await
        .context("When reading data json body")?;

    let (data, data_raw_str) = parse_json_str_with_fallback::<BmsTableData>(&data_text)
        .context("When parsing data json")?;

    Ok((
        BmsTable { header, data },
        BmsTableRaw {
            header_json_url,
            header_raw,
            data_json_url,
            data_raw: data_raw_str,
        },
    ))
}

/// Fetch and parse a complete BMS difficulty table.
///
/// See [`fetch_table_full`].
///
/// # Errors
///
/// Propagates network, parsing, and join errors from [`fetch_table_full`].
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
///
/// # Errors
///
/// Propagates network and parsing errors from [`fetch_table_list_full`].
pub async fn fetch_table_list(client: &Client, web_url: impl IntoUrl) -> Result<Vec<BmsTableInfo>> {
    let (out, _raw) = fetch_table_list_full(client, web_url)
        .await
        .context("When fetching table list full")?;
    Ok(out)
}

/// Fetch a list of BMS difficulty tables along with the raw JSON string.
///
/// Returns the parsed array of list entries and the raw JSON response text for recording or debugging.
///
/// # Errors
///
/// Returns an error if fetching or parsing the table list fails.
pub async fn fetch_table_list_full(
    client: &Client,
    web_url: impl IntoUrl,
) -> Result<(Vec<BmsTableInfo>, String)> {
    let list_url = web_url.into_url().context("When parsing table list url")?;
    let list_text = client
        .get(list_url)
        .send()
        .await
        .context("When fetching table list")?
        .text()
        .await
        .context("When reading table list body")?;

    let (list, raw_used) = parse_json_str_with_fallback::<BmsTableList>(&list_text)
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
///
/// # Errors
///
/// Returns an error when building the HTTP client fails.
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
