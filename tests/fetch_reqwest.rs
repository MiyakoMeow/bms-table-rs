#![cfg(feature = "reqwest")]
//! Unit tests for network fetching flow (requires the `reqwest` feature)
//!
//! Primarily checks error paths and robustness, e.g., errors for invalid URLs.

// Network-related tests (when the reqwest feature is enabled)

#[tokio::test]
async fn test_fetch_table_invalid_url() {
    let client = bms_table::fetch::reqwest::make_lenient_client().unwrap();
    let url = url::Url::parse("https://invalid-url-that-does-not-exist.com").unwrap();
    let result = bms_table::fetch::reqwest::fetch_table(&client, url).await;
    assert!(result.is_err());
}
