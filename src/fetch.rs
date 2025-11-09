//! 数据获取与 HTML 解析辅助模块
//!
//! 在启用 `scraper` 特性时提供 HTML 解析能力，用于从页面的
//! `<meta name="bmstable" content="...">` 中提取头部 JSON 的地址。
//! 同时提供一个统一的入口将响应字符串解析为头部 JSON 或其 URL。
//!
//! # 示例
//!
//! ```rust
//! # use bms_table::fetch::{get_web_header_json_value, HeaderQueryContent};
//! let html = r#"
//! <!DOCTYPE html>
//! <html>
//!   <head>
//!     <meta name="bmstable" content="header.json">
//!   </head>
//!   <body></body>
//! </html>
//! "#;
//! match get_web_header_json_value(html).unwrap() {
//!     HeaderQueryContent::Url(u) => assert_eq!(u, "header.json"),
//!     _ => unreachable!(),
//! }
//! ```
#![cfg(feature = "scraper")]

pub mod reqwest;

use anyhow::{Result, anyhow};
use scraper::{Html, Selector};
use serde_json::Value;

/// [`get_web_header_json_value`] 的返回类型。
///
/// - 当输入是 HTML 时，返回从 `<meta name="bmstable">` 提取的 URL；
/// - 当输入是 JSON 时，返回解析后的 JSON 值。
pub enum HeaderQueryContent {
    /// 提取到的头部 JSON 地址。
    ///
    /// 可能为相对或绝对 URL，建议使用 `url::Url::join` 进行拼接。
    Url(String),
    /// 原始头部 JSON 内容。
    Json(Value),
}

/// 将响应字符串解析为头部 JSON 或其 URL。
///
/// 解析策略：优先尝试按 JSON 解析；若失败则按 HTML 解析并提取 bmstable URL。
///
/// # 返回
///
/// - `HeaderQueryContent::Json`：输入是 JSON；
/// - `HeaderQueryContent::Url`：输入是 HTML。
///
/// # 错误
///
/// 当输入为 HTML 且未找到 bmstable 字段时返回错误。
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

/// 从 HTML 页面内容中提取 bmstable 字段指向的 JSON 文件 URL。
///
/// 该函数会扫描 `<meta>` 标签，寻找 `name="bmstable"` 的元素，并读取其 `content` 属性。
///
/// # 错误
///
/// 当未找到目标标签或 `content` 为空时返回错误。
pub fn extract_bmstable_url(html_content: &str) -> Result<String> {
    let document = Html::parse_document(html_content);

    // 查找所有meta标签
    let Ok(meta_selector) = Selector::parse("meta") else {
        return Err(anyhow!("未找到meta标签"));
    };

    for element in document.select(&meta_selector) {
        // 检查是否有name属性为"bmstable"的meta标签
        if let Some(name_attr) = element.value().attr("name")
            && name_attr == "bmstable"
        {
            // 获取content属性
            if let Some(content_attr) = element.value().attr("content")
                && !content_attr.is_empty()
            {
                return Ok(content_attr.to_string());
            }
        }
    }

    Err(anyhow!("未找到bmstable字段"))
}
