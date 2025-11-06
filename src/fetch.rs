//! BMS表格数据获取模块
//!
//! 这个模块提供了从BMS表格网站获取和解析数据的功能。
//! 支持从HTML页面提取bmstable字段，解析JSON格式的表格头信息和谱面数据。
//!
//! # 主要功能
//!
//! - 从HTML页面中提取bmstable字段指向的JSON文件URL
//! - 解析BMS表格头信息（包含课程、奖杯等元数据）
//! - 获取和解析谱面数据（包含歌曲信息、下载链接等）
//! - 完整的BMS表格数据获取流程

use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;

/// 基于 reqwest 的高层获取函数
#[cfg(feature = "reqwest")]
pub async fn fetch_bms_table(web_url: &str) -> Result<crate::BmsTable> {
    let web_url = Url::parse(web_url)?;
    let web_response = reqwest::Client::new()
        .get(web_url.clone())
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let (header_url, header_json) = match crate::get_web_header_json_value(&web_response)? {
        crate::HeaderQueryContent::Url(header_url_string) => {
            let header_url = web_url.join(&header_url_string)?;
            let header_response = reqwest::Client::new()
                .get(header_url.clone())
                .send()
                .await
                .map_err(|e| anyhow!("When fetching header: {e}"))?;
            let header_response_string = header_response
                .text()
                .await
                .map_err(|e| anyhow!("When parsing header response: {e}"))?;
            let crate::HeaderQueryContent::Json(header_json) =
                crate::get_web_header_json_value(&header_response_string)?
            else {
                return Err(anyhow!(
                    "Cycled header found. web_url: {web_url}, header_url: {header_url_string}"
                ));
            };
            (header_url, header_json)
        }
        crate::HeaderQueryContent::Json(value) => (web_url, value),
    };
    let data_url_str = header_json
        .get("data_url")
        .ok_or(anyhow!("\"data_url\" not found in header json!"))?
        .as_str()
        .ok_or(anyhow!("\"data_url\" is not a string!"))?;
    let data_url = header_url.join(data_url_str)?;
    let data_response = reqwest::Client::new()
        .get(data_url)
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let data_json: Value = serde_json::from_str(&data_response)?;
    // 直接使用库内反序列化生成 BmsTable
    let header: crate::BmsTableHeader = serde_json::from_value(header_json)
        .map_err(|e| anyhow!("When parsing header json: {e}"))?;
    let data: crate::BmsTableData =
        serde_json::from_value(data_json).map_err(|e| anyhow!("When parsing data json: {e}"))?;
    Ok(crate::BmsTable { header, data })
}

#[cfg(test)]
mod network_tests {
    #[cfg(feature = "reqwest")]
    #[tokio::test]
    async fn test_fetch_table_json_data_invalid_url() {
        let result = super::fetch_bms_table("https://invalid-url-that-does-not-exist.com").await;
        assert!(result.is_err());
    }

    #[cfg(feature = "reqwest")]
    #[tokio::test]
    async fn test_fetch_bms_table_invalid_url() {
        let result = super::fetch_bms_table("https://invalid-url-that-does-not-exist.com").await;
        assert!(result.is_err());
    }
}

/// 从HTML页面内容中，提取bmstable字段指向的JSON文件URL
pub fn extract_bmstable_url(html_content: &str) -> Result<String> {
    let document = Html::parse_document(html_content);

    // 查找所有meta标签
    let Ok(meta_selector) = Selector::parse("meta") else {
        return Err(anyhow!("未找到meta标签"));
    };

    for element in document.select(&meta_selector) {
        // 检查是否有name属性为"bmstable"的meta标签
        if let Some(name_attr) = element.value().attr("name") {
            if name_attr == "bmstable" {
                // 获取content属性
                if let Some(content_attr) = element.value().attr("content") {
                    if !content_attr.is_empty() {
                        return Ok(content_attr.to_string());
                    }
                }
            }
        }
    }

    Err(anyhow!("未找到bmstable字段"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试解析器创建功能
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

    /// 测试解析器处理没有bmstable字段的HTML
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

    /// 测试BmsTableHeader反序列化，支持Vec<CourseInfo>格式
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

        let result: Result<crate::BmsTableHeader, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.name, "Test Table");
        assert_eq!(header.symbol, "test");
        assert_eq!(header.data_url, "score.json");
        assert_eq!(header.course.len(), 1); // 外层长度为1
        assert_eq!(header.course[0].len(), 1); // 内层有1个课程
        assert_eq!(header.course[0][0].name, "Course 1");

        // 验证md5list被转换为charts
        let course = &header.course[0][0];
        assert_eq!(course.charts.len(), 2); // 两个md5值转换为两个charts
        assert_eq!(course.charts[0].md5, Some("abc123".to_string()));
        assert_eq!(course.charts[1].md5, Some("def456".to_string()));
    }

    /// 测试BmsTableHeader反序列化，支持Vec<Vec<CourseInfo>>格式
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

        let result: Result<crate::BmsTableHeader, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let header = result.unwrap();
        assert_eq!(header.name, "Test Table");
        assert_eq!(header.symbol, "test");
        assert_eq!(header.data_url, "score.json");
        assert_eq!(header.course.len(), 2); // 外层有2个课程组
        assert_eq!(header.course[0].len(), 1); // 第一个课程组有1个课程
        assert_eq!(header.course[1].len(), 1); // 第二个课程组有1个课程
        assert_eq!(header.course[0][0].name, "Course 1");
        assert_eq!(header.course[1][0].name, "Course 2");

        // 验证md5list被转换为charts
        let course1 = &header.course[0][0];
        assert_eq!(course1.charts.len(), 2); // 两个md5值转换为两个charts
        assert_eq!(course1.charts[0].md5, Some("abc123".to_string()));
        assert_eq!(course1.charts[1].md5, Some("def456".to_string()));

        let course2 = &header.course[1][0];
        assert_eq!(course2.charts.len(), 1); // 一个md5值转换为一个chart
        assert_eq!(course2.charts[0].md5, Some("ghi789".to_string()));
    }

    /// 测试CourseInfo反序列化，charts字段中ChartItem的level字段使用默认值
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

        let result: Result<crate::CourseInfo, _> = serde_json::from_str(json_data);
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

    /// 测试CourseInfo反序列化，sha256list转换为charts
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

        let result: Result<crate::CourseInfo, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let course_info = result.unwrap();
        assert_eq!(course_info.name, "Test Course");
        assert_eq!(course_info.constraint, vec!["grade_mirror"]);
        assert_eq!(course_info.trophy.len(), 1);
        assert_eq!(course_info.charts.len(), 2); // 两个sha256值转换为两个charts

        // 验证sha256list被转换为charts
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

    /// 测试CourseInfo反序列化，同时包含md5list和sha256list
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

        let result: Result<crate::CourseInfo, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let course_info = result.unwrap();
        assert_eq!(course_info.name, "Test Course");
        assert_eq!(course_info.constraint, vec!["grade_mirror"]);
        assert_eq!(course_info.trophy.len(), 1);
        assert_eq!(course_info.charts.len(), 3); // 1个现有chart + 1个md5 + 1个sha256

        // 验证现有chart保持不变
        assert_eq!(course_info.charts[0].level, "2");
        assert_eq!(
            course_info.charts[0].title,
            Some("Existing Chart".to_string())
        );
        assert_eq!(
            course_info.charts[0].artist,
            Some("Test Artist".to_string())
        );

        // 验证md5转换的chart
        assert_eq!(course_info.charts[1].md5, Some("md5_hash_1".to_string()));
        assert_eq!(course_info.charts[1].level, "0");

        // 验证sha256转换的chart
        assert_eq!(
            course_info.charts[2].sha256,
            Some("sha256_hash_1".to_string())
        );
        assert_eq!(course_info.charts[2].level, "0");
    }
}
