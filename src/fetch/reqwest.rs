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
//! use bms_table::fetch::reqwest::{fetch_table, make_lenient_client};
//! let client = make_lenient_client()?;
//! let table = fetch_table(&client, "https://stellabms.xyz/sl/table.html").await?;
//! assert!(!table.data.charts.is_empty());
//! # Ok(())
//! # }
//! ```
#![cfg(feature = "reqwest")]

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;
use url::Url;

use crate::{BmsTable, BmsTableInfo, BmsTableRaw};

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
        BmsTable { header, data },
        BmsTableRaw {
            header_raw,
            data_raw: data_response,
        },
    ))
}

/// 从网页或头部 JSON 源拉取并解析完整的 BMS 难度表。
///
/// 参考 [`fetch_table_full`]。
pub async fn fetch_table(client: &reqwest::Client, web_url: &str) -> Result<BmsTable> {
    let (table, _raw) = fetch_table_full(client, web_url).await?;
    Ok(table)
}

/// 获取 BMS 难度表列表。
///
/// 从提供的 `web_url` 下载 JSON 数组并解析为 [`crate::BmsTableInfo`] 列表。
/// 仅要求每个元素包含 `name`、`symbol` 与 `url`（字符串），其他字段将被收集到 `extra` 中。
pub async fn fetch_table_list(
    client: &reqwest::Client,
    web_url: &str,
) -> Result<Vec<BmsTableInfo>> {
    let (out, _raw) = fetch_table_list_full(client, web_url).await?;
    Ok(out)
}

/// 获取 BMS 难度表列表及其原始 JSON 字符串。
///
/// 返回解析后的列表项数组与响应的原始 JSON 文本，便于记录或调试。
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

    let value: Value = serde_json::from_str(&response_text)?;
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

/// 创建一个规则宽松、兼容性更强的 HTTP 客户端。
///
/// - 设置浏览器 UA；
/// - 配置超时与重定向；
/// - 接受无效证书（用于少数不规范站点）；
///
/// 注意：生产环境应审慎使用 `danger_accept_invalid_certs`。
pub fn make_lenient_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119 Safari/537.36 bms-table-rs")
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| anyhow!("When building client: {e}"))?;
    Ok(client)
}
