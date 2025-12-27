//! Unit tests for serialization behavior of header, chart items, and table data
#![cfg(feature = "serde")]

use bms_table::{BmsTableData, BmsTableHeader, ChartItem};
use std::collections::BTreeMap;

#[test]
fn test_header_serialize_flattens_extra() {
    let header = BmsTableHeader {
        name: "Test Table".to_string(),
        symbol: "tt".to_string(),
        data_url: "charts.json".to_string(),
        course: vec![Vec::new()],
        level_order: vec!["0".to_string(), "1".to_string()],
        extra: {
            let mut m = BTreeMap::new();
            m.insert("extra_field".to_string(), serde_json::json!("extra_value"));
            m.insert("another_field".to_string(), serde_json::json!(123));
            m
        },
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
        parsed.extra.get("extra_field"),
        Some(&serde_json::json!("extra_value"))
    );
    assert_eq!(
        parsed.extra.get("another_field"),
        Some(&serde_json::json!(123))
    );
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
        extra: {
            let mut m = BTreeMap::new();
            m.insert("custom_field".to_string(), serde_json::json!("value"));
            m.insert("rating".to_string(), serde_json::json!(4.5));
            m
        },
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
    assert_eq!(
        parsed.extra.get("custom_field"),
        Some(&serde_json::json!("value"))
    );
    assert_eq!(parsed.extra.get("rating"), Some(&serde_json::json!(4.5)));
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
        extra: BTreeMap::new(),
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
        extra: BTreeMap::new(),
    };
    let data = BmsTableData {
        charts: vec![item1, item2],
    };

    let value = serde_json::to_value(&data).unwrap();
    assert!(value.is_array());

    let parsed: BmsTableData = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.charts.len(), 2);
    let [c0, c1] = parsed.charts.as_slice() else {
        panic!(
            "expected two charts, got {}: {:?}",
            parsed.charts.len(),
            parsed.charts
        );
    };
    assert_eq!(c0.level.as_str(), "0");
    assert_eq!(c1.level.as_str(), "1");
}
