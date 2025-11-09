//! 基于 `reqwest` 的网络获取模块
//!
//! 提供一站式从网页或头部 JSON 源拉取并解析 BMS 难度表的能力：
//! - 获取网页并从 HTML 提取 bmstable 头部地址（如有）；
//! - 下载并解析头部 JSON；
//! - 根据头部中的 `data_url` 下载谱面数据并解析；
//! - 返回包含表头与谱面集合的 `BmsTable`。
//!
//! # 示例
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::fetch_bms_table;
//! let table = fetch_bms_table("https://stellabms.xyz/sl/table.html").await?;
//! assert!(!table.data.charts.is_empty());
//! # Ok(())
//! # }
//! ```
#![cfg(feature = "reqwest")]

use anyhow::{anyhow, Result};
use serde_json::Value;
use url::Url;

use crate::BmsTableRaw;

/// 从网页或头部 JSON 源拉取并解析完整的 BMS 难度表。
///
/// # 参数
///
/// - `web_url`：网页地址或直接指向头部 JSON 的地址。
///
/// # 返回
///
/// 解析后的 [`crate::BmsTable`]，包含表头与谱面数据。
///
/// # 错误
///
/// - 网络请求失败（连接失败、超时等）
/// - 响应内容无法解析为 HTML/JSON 或结构不符合预期
/// - 头部 JSON 未包含 `data_url` 字段或其类型不正确
pub async fn fetch_bms_table_full(web_url: &str) -> Result<(crate::BmsTable, BmsTableRaw)> {
    let web_url = Url::parse(web_url)?;
    let web_response = reqwest::Client::new()
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
                let header_response = reqwest::Client::new()
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
    let data_response = reqwest::Client::new()
        .get(data_url)
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let data_json: Value = serde_json::from_str(&data_response)?;
    // 直接使用库内反序列化生成 BmsTable
    let header: crate::BmsTableHeader = serde_json::from_value(header_json)
        .map_err(|e| anyhow!("When parsing header json: {e}"))?;
    let data: crate::BmsTableData =
        serde_json::from_value(data_json).map_err(|e| anyhow!("When parsing data json: {e}"))?;
    Ok((
        crate::BmsTable { header, data },
        BmsTableRaw {
            header_raw,
            data_raw: data_response,
        },
    ))
}

/// 从网页或头部 JSON 源拉取并解析完整的 BMS 难度表。
///
/// 参考 [`fetch_bms_table_full`]。
pub async fn fetch_bms_table(web_url: &str) -> Result<crate::BmsTable> {
    let (table, _raw) = fetch_bms_table_full(web_url).await?;
    Ok(table)
}
