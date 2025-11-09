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

    // 1) 优先从<meta name="bmstable" content="...">或<meta property="bmstable">提取
    for element in document.select(&meta_selector) {
        // name 或 property 为 bmstable 的标签
        let is_bmstable = element
            .value()
            .attr("name")
            .is_some_and(|v| v.eq_ignore_ascii_case("bmstable"))
            || element
                .value()
                .attr("property")
                .is_some_and(|v| v.eq_ignore_ascii_case("bmstable"));
        if is_bmstable
            && let Some(content_attr) = element.value().attr("content")
            && !content_attr.is_empty()
        {
            return Ok(content_attr.to_string());
        }
    }

    // 2) 其次尝试<link rel="bmstable" href="...json">
    if let Ok(link_selector) = Selector::parse("link") {
        for element in document.select(&link_selector) {
            let rel = element.value().attr("rel");
            let href = element.value().attr("href");
            if rel.is_some_and(|v| v.eq_ignore_ascii_case("bmstable"))
                && let Some(href) = href
                && !href.is_empty()
            {
                return Ok(href.to_string());
            }
        }
    }

    // 3) 再尝试在常见标签属性中寻找 *header*.json 线索
    //    - a[href], link[href], script[src], meta[content]
    let lower_contains_header_json = |s: &str| {
        let ls = s.to_ascii_lowercase();
        ls.contains("header") && ls.ends_with(".json")
    };

    // a[href]
    if let Ok(a_selector) = Selector::parse("a") {
        for element in document.select(&a_selector) {
            if let Some(href) = element.value().attr("href")
                && lower_contains_header_json(href)
            {
                return Ok(href.to_string());
            }
        }
    }

    // link[href]
    if let Ok(link_selector) = Selector::parse("link") {
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href")
                && lower_contains_header_json(href)
            {
                return Ok(href.to_string());
            }
        }
    }

    // script[src]
    if let Ok(script_selector) = Selector::parse("script") {
        for element in document.select(&script_selector) {
            if let Some(src) = element.value().attr("src")
                && lower_contains_header_json(src)
            {
                return Ok(src.to_string());
            }
        }
    }

    // meta[content]
    for element in document.select(&meta_selector) {
        if let Some(content_attr) = element.value().attr("content")
            && lower_contains_header_json(content_attr)
        {
            return Ok(content_attr.to_string());
        }
    }

    // 4) 最后进行原始文本的极简启发式搜索：匹配包含"header"且以 .json 结尾的子串
    if let Some((start, end)) = find_header_json_in_text(html_content) {
        let candidate = &html_content[start..end];
        return Ok(candidate.to_string());
    }

    Err(anyhow!("未找到bmstable字段或header JSON线索"))
}

/// 在原始文本中查找类似 "*header*.json" 的子串，返回起止下标（若找到）。
fn find_header_json_in_text(s: &str) -> Option<(usize, usize)> {
    let lower = s.to_ascii_lowercase();
    let mut pos = 0;
    while let Some(idx) = lower[pos..].find("header") {
        let global_idx = pos + idx;
        // 在 header 之后寻找 .json
        if let Some(json_rel) = lower[global_idx..].find(".json") {
            let end = global_idx + json_rel + ".json".len();
            // 试着往前找最近的引号或空白作为起点
            let start = lower[..global_idx]
                .rfind(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(global_idx);
            if end > start {
                return Some((start, end));
            }
        }
        pos = global_idx + 6; // 跳过 "header"
    }
    None
}
