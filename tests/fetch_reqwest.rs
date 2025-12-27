//! Unit tests for network fetching flow (requires the `reqwest` feature)
//!
//! Primarily checks error paths and robustness, e.g., errors for invalid URLs.
#![cfg(feature = "reqwest")]

// Network-related tests (when the reqwest feature is enabled)

#[tokio::test]
async fn test_fetch_table_invalid_url() {
    let fetcher = bms_table::fetch::reqwest::Fetcher::lenient().unwrap();
    let url = url::Url::parse("https://invalid-url-that-does-not-exist.com").unwrap();
    let result = fetcher.fetch_table(url).await;
    assert!(result.is_err());
}
