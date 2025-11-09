//! JSON 解析与数据结构反序列化的单元测试
//!
//! 覆盖表头、课程、谱面数据的常见与边界输入，确保反序列化与字段兼容行为正确。
#![cfg(feature = "serde")]

use bms_table::{BmsTable, BmsTableData, BmsTableHeader, CourseInfo};
use serde_json::json;
use std::collections::HashMap;

// JSON 解析相关测试：来自原 lib_tests.rs 与 fetch_tests.rs

#[test]
fn test_build_bms_table_from_json() {
    let header_json = json!({
        "name": "Test Table",
        "symbol": "test",
        "data_url": "charts.json",
        "course": [
            [
                {
                    "name": "Test Course",
                    "constraint": ["grade_mirror"],
                    "trophy": [
                        {
                            "name": "goldmedal",
                            "missrate": 1.0,
                            "scorerate": 90.0
                        }
                    ],
                    "md5": ["test_md5_1", "test_md5_2"]
                }
            ]
        ],
        "level_order": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, "!i"],
        "extra_field": "extra_value",
        "another_field": 123
    });
    let data_json = json!([
        {
            "level": "1",
            "id": 1,
            "md5": "test_md5_1",
            "sha256": "test_sha256_1",
            "title": "Test Song",
            "artist": "Test Artist",
            "url": "https://example.com/test.bms",
            "url_diff": "https://example.com/test_diff.bms",
            "custom_field": "custom_value",
            "rating": 5.0
        }
    ]);
    let header: BmsTableHeader = serde_json::from_value(header_json).unwrap();
    let data: BmsTableData = serde_json::from_value(data_json).unwrap();
    let bms_table = BmsTable { header, data };
    assert_eq!(bms_table.header.name, "Test Table");
    assert_eq!(bms_table.header.symbol, "test");
    assert_eq!(bms_table.header.data_url, "charts.json");
    assert_eq!(bms_table.header.course.len(), 1);
    assert_eq!(bms_table.data.charts.len(), 1);

    let course = &bms_table.header.course[0][0];
    assert_eq!(course.name, "Test Course");
    assert_eq!(course.constraint, vec!["grade_mirror"]);
    assert_eq!(course.trophy.len(), 1);
    assert_eq!(course.trophy[0].name, "goldmedal");
    assert_eq!(course.trophy[0].missrate, 1.0);
    assert_eq!(course.trophy[0].scorerate, 90.0);
    assert_eq!(course.charts.len(), 2);
    assert_eq!(course.charts[0].md5, Some("test_md5_1".to_string()));
    assert_eq!(course.charts[1].md5, Some("test_md5_2".to_string()));

    let score = &bms_table.data.charts[0];
    assert_eq!(score.level, "1");
    assert_eq!(score.md5, Some("test_md5_1".to_string()));
    assert_eq!(score.sha256, Some("test_sha256_1".to_string()));
    assert_eq!(score.title, Some("Test Song".to_string()));
    assert_eq!(score.artist, Some("Test Artist".to_string()));
    assert_eq!(score.url, Some("https://example.com/test.bms".to_string()));
    assert_eq!(
        score.url_diff,
        Some("https://example.com/test_diff.bms".to_string())
    );

    assert_eq!(
        bms_table.header.extra.get("extra_field"),
        Some(&json!("extra_value"))
    );
    assert_eq!(
        bms_table.header.extra.get("another_field"),
        Some(&json!(123))
    );
    assert!(!bms_table.header.extra.contains_key("name"));

    assert_eq!(
        score.extra.get("custom_field"),
        Some(&json!("custom_value"))
    );
    assert_eq!(score.extra.get("rating"), Some(&json!(5.0)));
    assert!(!score.extra.contains_key("level"));

    assert_eq!(bms_table.header.level_order.len(), 22);
    assert_eq!(bms_table.header.level_order[0], "0");
    assert_eq!(bms_table.header.level_order[20], "20");
    assert_eq!(bms_table.header.level_order[21], "!i");
    assert!(!bms_table.header.extra.contains_key("level_order"));
}

#[test]
fn test_build_bms_table_with_empty_fields() {
    let header_json = json!({
        "name": "Test Table",
        "symbol": "test",
        "data_url": "charts.json",
        "course": []
    });
    let data_json = json!([
        {
            "level": "1",
            "id": 1,
            "md5": "",
            "sha256": "",
            "title": "",
            "artist": "",
            "url": "",
            "url_diff": ""
        }
    ]);
    let header: BmsTableHeader = serde_json::from_value(header_json).unwrap();
    let data: BmsTableData = serde_json::from_value(data_json).unwrap();
    let bms_table = BmsTable { header, data };
    let score = &bms_table.data.charts[0];
    assert_eq!(score.level, "1");
    assert_eq!(score.md5, None);
    assert_eq!(score.sha256, None);
    assert_eq!(score.title, None);
    assert_eq!(score.artist, None);
    assert_eq!(score.url, None);
    assert_eq!(score.url_diff, None);
}

#[test]
fn test_bms_table_creation() {
    let header = BmsTableHeader {
        name: "Test Table".to_string(),
        symbol: "test".to_string(),
        data_url: "https://example.com/charts.json".to_string(),
        course: vec![],
        level_order: vec!["0".to_string(), "1".to_string()],
        extra: HashMap::new(),
    };
    let data = BmsTableData { charts: vec![] };
    let bms_table = BmsTable { header, data };

    assert_eq!(bms_table.header.name, "Test Table");
    assert_eq!(bms_table.header.symbol, "test");
    assert_eq!(bms_table.header.course.len(), 0);
    assert_eq!(bms_table.data.charts.len(), 0);
    assert_eq!(bms_table.header.level_order.len(), 2);
}

#[test]
fn test_bms_table_partial_eq() {
    let header1 = BmsTableHeader {
        name: "Test Table".to_string(),
        symbol: "test".to_string(),
        data_url: "https://example.com/charts.json".to_string(),
        course: vec![],
        level_order: vec!["0".to_string(), "1".to_string()],
        extra: HashMap::new(),
    };
    let data1 = BmsTableData { charts: vec![] };
    let table1 = BmsTable {
        header: header1.clone(),
        data: data1,
    };

    let header2 = header1;
    let data2 = BmsTableData { charts: vec![] };
    let table2 = BmsTable {
        header: header2,
        data: data2,
    };

    assert_eq!(table1, table2);
}

#[test]
fn test_build_bms_table_invalid_json() {
    let header_json = json!({
        "name": "Test Table",
        "symbol": "test",
        "data_url": "charts.json"
        // 缺少必要的字段
    });
    let data_json = json!([
        {
            "level": "1",
            "id": 1
            // 缺少必要的字段
        }
    ]);
    let header: BmsTableHeader = serde_json::from_value(header_json).unwrap();
    let data: BmsTableData = serde_json::from_value(data_json).unwrap();
    let _bms_table = BmsTable { header, data };
}

#[test]
fn test_bms_table_header_deserialize_vec_course_info() {
    let json_data = r#"{
        "name": "Test Table",
        "symbol": "test",
        "data_url": "score.json",
        "course": [
            {
                "name": "Course 1",
                "constraint": ["grade_mirror"],
                "trophy": [
                    {
                        "name": "goldmedal",
                        "missrate": 5.0,
                        "scorerate": 70.0
                    }
                ],
                "md5": ["abc123", "def456"]
            }
        ]
    }"#;

    let result: serde_json::Result<BmsTableHeader> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let header = result.unwrap();
    assert_eq!(header.name, "Test Table");
    assert_eq!(header.symbol, "test");
    assert_eq!(header.data_url, "score.json");
    assert_eq!(header.course.len(), 1);
    assert_eq!(header.course[0].len(), 1);
    assert_eq!(header.course[0][0].name, "Course 1");

    let course = &header.course[0][0];
    assert_eq!(course.charts.len(), 2);
    assert_eq!(course.charts[0].md5, Some("abc123".to_string()));
    assert_eq!(course.charts[1].md5, Some("def456".to_string()));
}

#[test]
fn test_bms_table_header_deserialize_vec_vec_course_info() {
    let json_data = r#"{
        "name": "Test Table",
        "symbol": "test",
        "data_url": "score.json",
        "course": [
            [
                {
                    "name": "Course 1",
                    "constraint": ["grade_mirror"],
                    "trophy": [
                        {
                            "name": "goldmedal",
                            "missrate": 5.0,
                            "scorerate": 70.0
                        }
                    ],
                    "md5": ["abc123", "def456"]
                }
            ],
            [
                {
                    "name": "Course 2",
                    "constraint": ["ln"],
                    "trophy": [
                        {
                            "name": "silvermedal",
                            "missrate": 10.0,
                            "scorerate": 60.0
                        }
                    ],
                    "md5": ["ghi789"]
                }
            ]
        ]
    }"#;

    let result: serde_json::Result<BmsTableHeader> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let header = result.unwrap();
    assert_eq!(header.name, "Test Table");
    assert_eq!(header.symbol, "test");
    assert_eq!(header.data_url, "score.json");
    assert_eq!(header.course.len(), 2);
    assert_eq!(header.course[0].len(), 1);
    assert_eq!(header.course[1].len(), 1);
    assert_eq!(header.course[0][0].name, "Course 1");
    assert_eq!(header.course[1][0].name, "Course 2");

    let course1 = &header.course[0][0];
    assert_eq!(course1.charts.len(), 2);
    assert_eq!(course1.charts[0].md5, Some("abc123".to_string()));
    assert_eq!(course1.charts[1].md5, Some("def456".to_string()));

    let course2 = &header.course[1][0];
    assert_eq!(course2.charts.len(), 1);
    assert_eq!(course2.charts[0].md5, Some("ghi789".to_string()));
}

#[test]
fn test_course_info_deserialize_charts_with_default_level() {
    let json_data = r#"{
        "name": "Test Course",
        "constraint": ["grade_mirror"],
        "trophy": [
            {
                "name": "goldmedal",
                "missrate": 5.0,
                "scorerate": 70.0
            }
        ],
        "charts": [
            {
                "title": "Test Song",
                "artist": "Test Artist",
                "url": "https://example.com/test.bms"
            },
            {
                "level": "1",
                "title": "Test Song 2",
                "artist": "Test Artist 2",
                "url": "https://example.com/test2.bms"
            }
        ]
    }"#;

    let result: serde_json::Result<CourseInfo> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let course_info = result.unwrap();
    assert_eq!(course_info.name, "Test Course");
    assert_eq!(course_info.constraint, vec!["grade_mirror"]);
    assert_eq!(course_info.trophy.len(), 1);
    assert_eq!(course_info.charts.len(), 2);

    let first_chart = &course_info.charts[0];
    assert_eq!(first_chart.level, "0");
    assert_eq!(first_chart.title, Some("Test Song".to_string()));
    assert_eq!(first_chart.artist, Some("Test Artist".to_string()));

    let second_chart = &course_info.charts[1];
    assert_eq!(second_chart.level, "1");
    assert_eq!(second_chart.title, Some("Test Song 2".to_string()));
    assert_eq!(second_chart.artist, Some("Test Artist 2".to_string()));
}

#[test]
fn test_course_info_deserialize_sha256list_to_charts() {
    let json_data = r#"{
        "name": "Test Course",
        "constraint": ["grade_mirror"],
        "trophy": [
            {
                "name": "goldmedal",
                "missrate": 5.0,
                "scorerate": 70.0
            }
        ],
        "sha256": ["sha256_hash_1", "sha256_hash_2"]
    }"#;

    let result: serde_json::Result<CourseInfo> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let course_info = result.unwrap();
    assert_eq!(course_info.name, "Test Course");
    assert_eq!(course_info.constraint, vec!["grade_mirror"]);
    assert_eq!(course_info.trophy.len(), 1);
    assert_eq!(course_info.charts.len(), 2);

    assert_eq!(
        course_info.charts[0].sha256,
        Some("sha256_hash_1".to_string())
    );
    assert_eq!(
        course_info.charts[1].sha256,
        Some("sha256_hash_2".to_string())
    );
    assert_eq!(course_info.charts[0].md5, None);
    assert_eq!(course_info.charts[1].md5, None);
}

#[test]
fn test_course_info_deserialize_md5_and_sha256_to_charts() {
    let json_data = r#"{
        "name": "Test Course",
        "constraint": ["grade_mirror"],
        "trophy": [
            {
                "name": "goldmedal",
                "missrate": 5.0,
                "scorerate": 70.0
            }
        ],
        "md5": ["md5_hash_1"],
        "sha256": ["sha256_hash_1"],
        "charts": [
            {
                "level": "2",
                "title": "Existing Chart",
                "artist": "Test Artist"
            }
        ]
    }"#;

    let result: serde_json::Result<CourseInfo> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let course_info = result.unwrap();
    assert_eq!(course_info.name, "Test Course");
    assert_eq!(course_info.constraint, vec!["grade_mirror"]);
    assert_eq!(course_info.trophy.len(), 1);
    assert_eq!(course_info.charts.len(), 3);

    assert_eq!(course_info.charts[0].level, "2");
    assert_eq!(
        course_info.charts[0].title,
        Some("Existing Chart".to_string())
    );
    assert_eq!(
        course_info.charts[0].artist,
        Some("Test Artist".to_string())
    );

    assert_eq!(course_info.charts[1].md5, Some("md5_hash_1".to_string()));
    assert_eq!(course_info.charts[1].level, "0");

    assert_eq!(
        course_info.charts[2].sha256,
        Some("sha256_hash_1".to_string())
    );
    assert_eq!(course_info.charts[2].level, "0");
}

#[test]
fn test_json_serialization() {
    // 测试库内的BmsTableHeader序列化/反序列化
    let header = bms_table::BmsTableHeader {
        name: "Test Table".to_string(),
        symbol: "test".to_string(),
        data_url: "charts.json".to_string(),
        course: vec![],
        level_order: vec!["0".to_string(), "1".to_string(), "!i".to_string()],
        extra: HashMap::new(),
    };

    let json = serde_json::to_string(&header).unwrap();
    let parsed: bms_table::BmsTableHeader = serde_json::from_str(&json).unwrap();
    let mut expected = header;
    // 反序列化逻辑：空的 `course: []` 视为扁平形式并包一层空组
    expected.course = vec![Vec::new()];
    assert_eq!(expected, parsed);
}
