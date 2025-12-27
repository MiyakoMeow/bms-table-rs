//! Unit tests for serialization of `BmsTableInfo` and `BmsTableList`
#![cfg(feature = "scraper")]
use std::collections::BTreeMap;

use bms_table::{BmsTableInfo, BmsTableList};
use url::Url;

#[test]
fn test_bms_table_list_serialize_array() {
    let item1 = BmsTableInfo {
        name: ".WAS難易度表".to_string(),
        symbol: "．".to_string(),
        url: Url::parse("https://darksabun.club/table/archive/was/").unwrap(),
        extra: {
            let mut m = BTreeMap::new();
            m.insert("tag1".to_string(), serde_json::json!("SP"));
            m.insert(
                "tag2".to_string(),
                serde_json::json!("Self-made Chart Only"),
            );
            m.insert(
                "comment".to_string(),
                serde_json::json!("Converted by Ribbit"),
            );
            m.insert("date".to_string(), serde_json::json!(""));
            m.insert("state".to_string(), serde_json::json!(""));
            m.insert("tag_order".to_string(), serde_json::json!("1"));
            m
        },
    };
    let item2 = BmsTableInfo {
        name: "[F]".to_string(),
        symbol: "[F]".to_string(),
        url: Url::parse("https://bms.hexlataia.xyz/tables/convert/%5BF%5D/table.html").unwrap(),
        extra: {
            let mut m = BTreeMap::new();
            m.insert("tag1".to_string(), serde_json::json!("SP"));
            m.insert(
                "tag2".to_string(),
                serde_json::json!("Self-made Chart Only"),
            );
            m.insert("comment".to_string(), serde_json::json!("Converted by Hex"));
            m.insert("date".to_string(), serde_json::json!(""));
            m.insert("state".to_string(), serde_json::json!(""));
            m.insert("tag_order".to_string(), serde_json::json!("1"));
            m
        },
    };
    let list = BmsTableList {
        listes: vec![item1, item2],
    };

    let value = serde_json::to_value(&list).unwrap();
    assert!(value.is_array());

    let parsed: BmsTableList = serde_json::from_value(value).unwrap();
    assert_eq!(parsed.listes.len(), 2);
    let [i0, i1] = parsed.listes.as_slice() else {
        panic!(
            "expected two items, got {}: {:?}",
            parsed.listes.len(),
            parsed.listes
        );
    };
    assert_eq!(i0.name.as_str(), ".WAS難易度表");
    assert_eq!(i1.symbol.as_str(), "[F]");
}
