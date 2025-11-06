use bms_table::fetch::extract_bmstable_url;
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
