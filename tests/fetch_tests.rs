use bms_table::fetch::extract_bmstable_url;
use serde_json;

// 来自 src/fetch/reqwest.rs 的解析与反序列化相关测试迁移

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

    let result: serde_json::Result<bms_table::BmsTableHeader> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let header = result.unwrap();
    assert_eq!(header.name, "Test Table");
    assert_eq!(header.symbol, "test");
    assert_eq!(header.data_url, "score.json");
    assert_eq!(header.course.len(), 1);
    assert_eq!(header.course[0].len(), 1);
    assert_eq!(header.course[0][0].name, "Course 1");

    // 验证md5list被转换为charts
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

    let result: serde_json::Result<bms_table::BmsTableHeader> = serde_json::from_str(json_data);
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

    // 验证md5list被转换为charts
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

    let result: serde_json::Result<bms_table::CourseInfo> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let course_info = result.unwrap();
    assert_eq!(course_info.name, "Test Course");
    assert_eq!(course_info.constraint, vec!["grade_mirror"]);
    assert_eq!(course_info.trophy.len(), 1);
    assert_eq!(course_info.charts.len(), 2);

    // 第一个chart没有level字段，应该使用默认值"0"
    let first_chart = &course_info.charts[0];
    assert_eq!(first_chart.level, "0");
    assert_eq!(first_chart.title, Some("Test Song".to_string()));
    assert_eq!(first_chart.artist, Some("Test Artist".to_string()));

    // 第二个chart有level字段，应该保持原值
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

    let result: serde_json::Result<bms_table::CourseInfo> = serde_json::from_str(json_data);
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

    let result: serde_json::Result<bms_table::CourseInfo> = serde_json::from_str(json_data);
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

// 网络相关测试迁移（保持特性门控）
#[cfg(feature = "reqwest")]
#[tokio::test]
async fn test_fetch_table_json_data_invalid_url() {
    let result =
        bms_table::fetch::reqwest::fetch_bms_table("https://invalid-url-that-does-not-exist.com")
            .await;
    assert!(result.is_err());
}

#[cfg(feature = "reqwest")]
#[tokio::test]
async fn test_fetch_bms_table_invalid_url() {
    let result =
        bms_table::fetch::reqwest::fetch_bms_table("https://invalid-url-that-does-not-exist.com")
            .await;
    assert!(result.is_err());
}
