//! BMS表格数据获取模块
//!
//! 这个模块提供了从BMS表格网站获取和解析数据的功能。
//! 支持从HTML页面提取bmstable字段，解析JSON格式的表格头信息和谱面数据。
//!
//! # 主要功能
//!
//! - 从HTML页面中提取bmstable字段指向的JSON文件URL
//! - 解析BMS表格头信息（包含课程、奖杯等元数据）
//! - 获取和解析谱面数据（包含歌曲信息、下载链接等）
//! - 完整的BMS表格数据获取流程
#![cfg(feature = "reqwest")]

use anyhow::{anyhow, Result};
use serde_json::Value;
use url::Url;

/// 基于 reqwest 的高层获取函数
pub async fn fetch_bms_table(web_url: &str) -> Result<crate::BmsTable> {
    let web_url = Url::parse(web_url)?;
    let web_response = reqwest::Client::new()
        .get(web_url.clone())
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let (header_url, header_json) = match crate::fetch::get_web_header_json_value(&web_response)? {
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
            (header_url, header_json)
        }
        crate::fetch::HeaderQueryContent::Json(value) => (web_url, value),
    };
    let data_url_str = header_json
        .get("data_url")
        .ok_or(anyhow!("\"data_url\" not found in header json!"))?
        .as_str()
        .ok_or(anyhow!("\"data_url\" is not a string!"))?;
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
    Ok(crate::BmsTable { header, data })
}
