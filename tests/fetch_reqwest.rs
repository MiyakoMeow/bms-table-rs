#![cfg(feature = "reqwest")]
//! 网络获取流程的单元测试（需启用 `reqwest` 特性）
//!
//! 主要检查错误路径与鲁棒性，例如无效地址的错误返回。

// 网络相关测试（reqwest 特性开启时）

#[tokio::test]
async fn test_fetch_table_invalid_url() {
    let client = bms_table::fetch::reqwest::make_lenient_client().unwrap();
    let result = bms_table::fetch::reqwest::fetch_table(
        &client,
        "https://invalid-url-that-does-not-exist.com",
    )
    .await;
    assert!(result.is_err());
}
