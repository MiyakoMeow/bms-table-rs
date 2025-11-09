#![cfg(feature = "serde")]

use bms_table::{BmsTableData, BmsTableHeader, ChartItem};
use bms_table::{BmsTableIndex, BmsTableIndexItem};
use url::Url;

#[test]
fn test_header_serialize_flattens_extra() {
    let header = BmsTableHeader {
        name: "Test Table".to_string(),
        symbol: "tt".to_string(),
        data_url: "charts.json".to_string(),
        course: vec![Vec::new()],
        level_order: vec!["0".to_string(), "1".to_string()],
        extra: serde_json::json!({
            "extra_field": "extra_value",
            "another_field": 123
        }),
    };

    let value = serde_json::to_value(&header).unwrap();
    let obj = value.as_object().expect("header must serialize to object");
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("symbol"));
    assert!(obj.contains_key("data_url"));
    assert!(obj.contains_key("course"));
    assert!(obj.contains_key("level_order"));
    assert!(obj.contains_key("extra_field"));
    assert!(obj.contains_key("another_field"));
    assert!(!obj.contains_key("extra"));

    let parsed: BmsTableHeader = serde_json::from_value(value).unwrap();
    assert_eq!(
        parsed.extra["extra_field"],
        serde_json::json!("extra_value")
    );
    assert_eq!(parsed.extra["another_field"], serde_json::json!(123));
}

#[test]
fn test_chart_item_serialize_flattens_extra() {
    let item = ChartItem {
        level: "1".to_string(),
        md5: Some("md5hash".to_string()),
        sha256: None,
        title: Some("Song Title".to_string()),
        subtitle: None,
        artist: None,
        subartist: None,
        url: Some("http://example.com".to_string()),
        url_diff: None,
        extra: serde_json::json!({
            "custom_field": "value",
            "rating": 4.5
        }),
    };

    let value = serde_json::to_value(&item).unwrap();
    let obj = value
        .as_object()
        .expect("chart item must serialize to object");
    assert!(obj.contains_key("level"));
    assert!(obj.contains_key("custom_field"));
    assert!(obj.contains_key("rating"));
    assert!(!obj.contains_key("extra"));

    let parsed: ChartItem = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.extra["custom_field"], serde_json::json!("value"));
    assert_eq!(parsed.extra["rating"], serde_json::json!(4.5));
}

#[test]
fn test_bms_table_data_serialize_array() {
    let item1 = ChartItem {
        level: "0".to_string(),
        md5: None,
        sha256: None,
        title: None,
        subtitle: None,
        artist: None,
        subartist: None,
        url: None,
        url_diff: None,
        extra: serde_json::json!({}),
    };
    let item2 = ChartItem {
        level: "1".to_string(),
        md5: None,
        sha256: None,
        title: None,
        subtitle: None,
        artist: None,
        subartist: None,
        url: None,
        url_diff: None,
        extra: serde_json::json!({}),
    };
    let data = BmsTableData {
        charts: vec![item1, item2],
    };

    let value = serde_json::to_value(&data).unwrap();
    assert!(value.is_array());

    let parsed: BmsTableData = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.charts.len(), 2);
    assert_eq!(parsed.charts[0].level, "0");
    assert_eq!(parsed.charts[1].level, "1");
}

#[test]
fn test_bms_table_index_serialize_array() {
    let item1 = BmsTableIndexItem {
        name: ".WAS難易度表".to_string(),
        symbol: "．".to_string(),
        url: Url::parse("https://darksabun.club/table/archive/was/").unwrap(),
        extra: serde_json::json!({
            "tag1": "SP",
            "tag2": "Self-made Chart Only",
            "comment": "Converted by Ribbit",
            "date": "",
            "state": "",
            "tag_order": "1"
        }),
    };
    let item2 = BmsTableIndexItem {
        name: "[F]".to_string(),
        symbol: "[F]".to_string(),
        url: Url::parse("https://bms.hexlataia.xyz/tables/convert/%5BF%5D/table.html").unwrap(),
        extra: serde_json::json!({
            "tag1": "SP",
            "tag2": "Self-made Chart Only",
            "comment": "Converted by Hex",
            "date": "",
            "state": "",
            "tag_order": "1"
        }),
    };
    let index = BmsTableIndex {
        indexes: vec![item1, item2],
    };

    let value = serde_json::to_value(&index).unwrap();
    assert!(value.is_array());

    let parsed: BmsTableIndex = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.indexes.len(), 2);
    assert_eq!(parsed.indexes[0].name, ".WAS難易度表");
    assert_eq!(parsed.indexes[1].symbol, "[F]");
}
