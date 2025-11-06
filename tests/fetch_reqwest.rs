#![cfg(feature = "reqwest")]

// 网络相关测试（reqwest 特性开启时）

#[tokio::test]
async fn test_fetch_bms_table_invalid_url() {
    let result =
        bms_table::fetch::reqwest::fetch_bms_table("https://invalid-url-that-does-not-exist.com")
            .await;
    assert!(result.is_err());
}
