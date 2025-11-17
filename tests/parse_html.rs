#![cfg(feature = "scraper")]
//! Unit tests for HTML parsing and bmstable URL extraction
//!
//! Verifies reading the `content` from `<meta name="bmstable">` and joining relative URLs.

use bms_table::fetch::{
    HeaderQueryContent, get_web_header_json_value, try_extract_bmstable_from_html,
};
use url::Url;

// Tests for HTML parsing and URL behavior

#[test]
fn test_parser_creation() {
    // Test extracting bmstable URL from HTML content
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

    let result = try_extract_bmstable_from_html(html_content);
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

    let result = try_extract_bmstable_from_html(html_content);
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
    match get_web_header_json_value::<serde_json::Value>(input).unwrap() {
        HeaderQueryContent::Value(v) => {
            assert_eq!(
                v.get("data_url").and_then(|x| x.as_str()),
                Some("charts.json")
            );
        }
        _ => panic!("should parse as JSON"),
    }
}
