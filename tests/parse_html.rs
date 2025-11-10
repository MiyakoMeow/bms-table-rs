#![cfg(feature = "scraper")]
//! HTML 解析与 bmstable URL 提取的单元测试
//!
//! 验证从 `<meta name="bmstable">` 中读取 `content` 的逻辑，以及相对地址的拼接。

use bms_table::fetch::extract_bmstable_url;
use bms_table::fetch::{HeaderQueryContent, get_web_header_json_value};
use url::Url;

// HTML 解析与 URL 相关测试

#[test]
fn test_parser_creation() {
    // 测试HTML内容中提取bmstable URL
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta name="bmstable" content="header.json">
    </head>
    <body>
        <h1>BMS Table</h1>
    </body>
    </html>
    "#;

    let result = extract_bmstable_url(html_content);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "header.json");
}

#[test]
fn test_parser_no_bmstable() {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>BMS Table</title>
    </head>
    <body>
        <h1>BMS Table</h1>
    </body>
    </html>
    "#;

    let result = extract_bmstable_url(html_content);
    assert!(result.is_err());
}

#[test]
fn test_url_parsing() {
    let base_url = "https://example.com/table.html";
    let bmstable_url = "header.json";

    let base_url_obj = Url::parse(base_url).unwrap();
    let header_url = base_url_obj.join(bmstable_url).unwrap();

    assert_eq!(header_url.as_str(), "https://example.com/header.json");
}

#[test]
fn test_get_web_header_json_value_parses_json_with_control_chars() {
    let input = "\u{0000}{\"data_url\":\"charts.json\"}\u{000C}";
    match get_web_header_json_value(input).unwrap() {
        HeaderQueryContent::Json(v) => {
            assert_eq!(
                v.get("data_url").and_then(|x| x.as_str()),
                Some("charts.json")
            );
        }
        _ => panic!("should parse as JSON"),
    }
}
