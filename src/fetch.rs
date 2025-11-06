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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// BMS表格头信息
///
/// 包含表格的基本信息和课程配置。
/// 这个结构体对应BMS表格头JSON文件的主要结构。
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BmsTableHeader {
    /// 表格名称，如 "Satellite"
    pub name: String,
    /// 表格符号，如 "sl"
    pub symbol: String,
    /// 谱面数据文件的相对URL，如 "score.json"
    pub data_url: String,
    /// 课程信息数组，每个元素是一个课程组的数组
    #[serde(default)]
    pub course: Vec<Vec<crate::CourseInfo>>,
    /// 难度等级顺序，包含数字和字符串
    #[serde(default)]
    pub level_order: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for BmsTableHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct BmsTableHeaderHelper {
            name: String,
            symbol: String,
            data_url: String,
            #[serde(default)]
            course: Value,
            #[serde(default)]
            level_order: Option<Vec<Value>>,
        }

        let helper = BmsTableHeaderHelper::deserialize(deserializer)?;

        // 处理course字段，支持Vec<CourseInfo>和Vec<Vec<CourseInfo>>两种格式
        let course = match &helper.course {
            Value::Array(arr) if !arr.is_empty() => {
                // 检查第一个元素是否为数组
                if let Some(Value::Array(_)) = arr.first() {
                    // 已经是Vec<Vec<CourseInfo>>格式
                    serde_json::from_value::<Vec<Vec<crate::CourseInfo>>>(helper.course.clone())
                        .map_err(serde::de::Error::custom)?
                } else {
                    // 是Vec<CourseInfo>格式，需要包装成Vec<Vec<CourseInfo>>
                    let course_info_vec: Vec<crate::CourseInfo> =
                        serde_json::from_value(helper.course.clone())
                            .map_err(serde::de::Error::custom)?;
                    vec![course_info_vec]
                }
            }
            _ => Vec::new(),
        };

        // 处理level_order，将数字和字符串都转换为字符串
        let level_order = helper
            .level_order
            .unwrap_or_default()
            .into_iter()
            .map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s,
                _ => v.to_string(),
            })
            .collect();

        Ok(Self {
            name: helper.name,
            symbol: helper.symbol,
            data_url: helper.data_url,
            course,
            level_order,
        })
    }
}

/// 奖杯信息
///
/// 定义了获得特定奖杯需要达到的谱面要求。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Trophy {
    /// 奖杯名称，如 "silvermedal" 或 "goldmedal"
    pub name: String,
    /// 最大miss率（百分比），如 5.0 表示最大5%的miss率
    pub missrate: f64,
    /// 最小得分率（百分比），如 70.0 表示至少70%的得分率
    pub scorerate: f64,
}

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
    crate::create_bms_table_from_json(header_url.as_str(), header_json, data_json)
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

/// 判断内容是否为JSON格式
#[allow(dead_code)]
pub fn is_json_content(content: &str) -> bool {
    content.trim().starts_with('{') || content.trim().starts_with('[')
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

        let result: Result<BmsTableHeader, _> = serde_json::from_str(json_data);
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

        let result: Result<BmsTableHeader, _> = serde_json::from_str(json_data);
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
