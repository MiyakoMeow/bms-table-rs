//! 网络请求与解析相关功能

pub mod reqwest;

use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde_json::Value;

/// [`get_web_header_json_value`]的返回类型
pub enum HeaderQueryContent {
    /// 注意：可能解析出相对或绝对Url，建议使用`url::Url::join`。
    Url(String),
    /// Json树
    Json(Value),
}

/// 从相应数据中提取Json树（Json内容）或Header地址（HTML内容）
pub fn get_web_header_json_value(response_str: &str) -> anyhow::Result<HeaderQueryContent> {
    // 先尝试按 JSON 解析；失败则当作 HTML 提取 bmstable URL
    match serde_json::from_str::<Value>(response_str) {
        Ok(header_json) => Ok(HeaderQueryContent::Json(header_json)),
        Err(_) => {
            let bmstable_url = extract_bmstable_url(response_str)?;
            Ok(HeaderQueryContent::Url(bmstable_url))
        }
    }
}

/// 从HTML页面内容中，提取bmstable字段指向的JSON文件URL
pub fn extract_bmstable_url(html_content: &str) -> Result<String> {
    let document = Html::parse_document(html_content);

    // 查找所有meta标签
    let Ok(meta_selector) = Selector::parse("meta") else {
        return Err(anyhow!("未找到meta标签"));
    };

    for element in document.select(&meta_selector) {
        // 检查是否有name属性为"bmstable"的meta标签
        if let Some(name_attr) = element.value().attr("name") {
            if name_attr == "bmstable" {
                // 获取content属性
                if let Some(content_attr) = element.value().attr("content") {
                    if !content_attr.is_empty() {
                        return Ok(content_attr.to_string());
                    }
                }
            }
        }
    }

    Err(anyhow!("未找到bmstable字段"))
}
