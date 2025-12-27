//! Network fetching module based on `reqwest`
//!
//! Provides an all-in-one ability to fetch and parse BMS difficulty tables from a web page or a header JSON source:
//! - Fetch the page and extract the bmstable header URL from HTML (if present);
//! - Download and parse the header JSON;
//! - Download and parse chart data according to `data_url` in the header;
//! - Return a parsed `BmsTable` plus the raw JSON strings used for parsing.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::Fetcher;
//! let fetcher = Fetcher::lenient()?;
//! let table = fetcher.fetch_table("https://stellabms.xyz/sl/table.html").await?.table;
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
use serde::de::DeserializeOwned;

use crate::{
    BmsTable, BmsTableData, BmsTableHeader, BmsTableList, BmsTableRaw,
    fetch::{
        FetchedTable, FetchedTableList, HeaderQueryContent, TableFetcher,
        header_query_with_fallback, parse_json_str_with_fallback,
    },
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
    /// Returns an error if fetching or parsing the table fails.
    pub async fn fetch_table(&self, web_url: impl IntoUrl) -> Result<FetchedTable> {
        let web_url = web_url.into_url().context("When parsing target url")?;

        let web_page_text = self.fetch_text(web_url.clone(), "web page").await?;

        let (web_header_query, web_used_text) =
            header_query_with_fallback::<BmsTableHeader>(&web_page_text)
                .context("When extracting header query from web page")?;

        let (header_json_url, header, header_raw) = match web_header_query {
            HeaderQueryContent::Url(header_url_string) => {
                let header_json_url = web_url
                    .join(&header_url_string)
                    .context("When resolving header json url")?;

                let header_text = self
                    .fetch_text(header_json_url.clone(), "header json")
                    .await?;

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
            HeaderQueryContent::Value(header) => (web_url, header, web_used_text),
        };

        let data_json_url = header_json_url
            .join(&header.data_url)
            .context("When resolving data json url")?;

        let (data, data_raw) = self
            .fetch_json_with_fallback::<BmsTableData>(
                data_json_url.clone(),
                "data json",
                "data json",
            )
            .await?;

        Ok(FetchedTable {
            table: BmsTable { header, data },
            raw: BmsTableRaw {
                header_json_url,
                header_raw,
                data_json_url,
                data_raw,
            },
        })
    }

    /// Fetch a list of BMS difficulty tables.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching or parsing the list fails.
    pub async fn fetch_table_list(&self, web_url: impl IntoUrl) -> Result<FetchedTableList> {
        let list_url = web_url.into_url().context("When parsing table list url")?;

        let (list, raw_used) = self
            .fetch_json_with_fallback::<BmsTableList>(list_url, "table list", "table list json")
            .await?;
        Ok(FetchedTableList {
            tables: list.listes,
            raw_json: raw_used,
        })
    }

    /// Fetch a URL as text, attaching contextual error messages.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the body cannot be read as text.
    async fn fetch_text(&self, url: reqwest::Url, fetch_ctx: &'static str) -> Result<String> {
        self.client
            .get(url)
            .send()
            .await
            .with_context(|| format!("When fetching {fetch_ctx}"))?
            .text()
            .await
            .with_context(|| format!("When reading {fetch_ctx} body"))
    }

    /// Fetch a URL and parse JSON with a control-character cleaning fallback.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching fails, or the response cannot be parsed as JSON.
    async fn fetch_json_with_fallback<T: DeserializeOwned>(
        &self,
        url: reqwest::Url,
        fetch_ctx: &'static str,
        parse_ctx: &'static str,
    ) -> Result<(T, String)> {
        let text = self.fetch_text(url, fetch_ctx).await?;
        parse_json_str_with_fallback::<T>(&text)
            .with_context(|| format!("When parsing {parse_ctx}"))
    }
}

impl TableFetcher for Fetcher {
    async fn fetch_table(&self, web_url: url::Url) -> Result<FetchedTable> {
        Fetcher::fetch_table(self, web_url).await
    }

    async fn fetch_table_list(&self, web_url: url::Url) -> Result<FetchedTableList> {
        Fetcher::fetch_table_list(self, web_url).await
    }
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
fn make_lenient_client() -> Result<Client> {
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
