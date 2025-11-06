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
    pub course: Vec<Vec<CourseInfo>>,
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
                    serde_json::from_value(helper.course.clone())
                        .map_err(serde::de::Error::custom)?
                } else {
                    // 是Vec<CourseInfo>格式，需要包装成Vec<Vec<CourseInfo>>
                    let course_info_vec: Vec<CourseInfo> =
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

/// 课程信息
///
/// 定义了一个BMS课程的所有相关信息，包括约束条件、奖杯要求和谱面数据。
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CourseInfo {
    /// 课程名称，如 "Satellite Skill Analyzer 2nd sl0"
    pub name: String,
    /// 约束条件列表，如 ["grade_mirror", "gauge_lr2", "ln"]
    #[serde(default)]
    pub constraint: Vec<String>,
    /// 奖杯信息列表，定义不同等级的奖杯要求
    #[serde(default)]
    pub trophy: Vec<Trophy>,
    /// 谱面数据列表，包含该课程的所有谱面信息
    #[serde(default)]
    pub charts: Vec<ChartItem>,
}

impl<'de> serde::Deserialize<'de> for CourseInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct CourseInfoHelper {
            name: String,
            #[serde(default)]
            constraint: Vec<String>,
            #[serde(default)]
            trophy: Vec<Trophy>,
            #[serde(default, rename = "md5")]
            md5list: Vec<String>,
            #[serde(default, rename = "sha256")]
            sha256list: Vec<String>,
            #[serde(default)]
            charts: Vec<Value>,
        }

        let helper = CourseInfoHelper::deserialize(deserializer)?;

        // 处理charts字段，为ChartItem的level字段设置默认值
        let mut charts = helper
            .charts
            .into_iter()
            .map(|mut chart_value| {
                if chart_value.get("level").is_none() {
                    // 如果level字段不存在，添加默认值0
                    let Some(chart_obj) = chart_value.as_object() else {
                        return Err(serde::de::Error::custom("chart_value is not an object"));
                    };
                    let mut chart_obj = chart_obj.clone();
                    chart_obj.insert(
                        "level".to_string(),
                        serde_json::Value::String("0".to_string()),
                    );
                    chart_value = serde_json::Value::Object(chart_obj);
                }
                serde_json::from_value(chart_value)
            })
            .collect::<Result<Vec<ChartItem>, serde_json::Error>>()
            .map_err(serde::de::Error::custom)?;

        // 将md5list转换为charts
        for md5 in &helper.md5list {
            charts.push(ChartItem {
                level: "0".to_string(),
                md5: Some(md5.clone()),
                sha256: None,
                title: None,
                subtitle: None,
                artist: None,
                subartist: None,
                url: None,
                url_diff: None,
                extra: serde_json::Value::Object(serde_json::Map::new()),
            });
        }

        // 将sha256list转换为charts
        for sha256 in &helper.sha256list {
            charts.push(ChartItem {
                level: "0".to_string(),
                md5: None,
                sha256: Some(sha256.clone()),
                title: None,
                subtitle: None,
                artist: None,
                subartist: None,
                url: None,
                url_diff: None,
                extra: serde_json::Value::Object(serde_json::Map::new()),
            });
        }

        Ok(Self {
            name: helper.name,
            constraint: helper.constraint,
            trophy: helper.trophy,
            charts,
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

/// 谱面数据项
///
/// 表示一个BMS文件的谱面数据，包含文件信息和下载链接。
/// 所有字段都是可选的，因为不同的BMS表格可能有不同的字段。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChartItem {
    /// 难度等级，如 "0"
    pub level: String,
    /// 文件的MD5哈希值
    pub md5: Option<String>,
    /// 文件的SHA256哈希值
    pub sha256: Option<String>,
    /// 歌曲标题
    pub title: Option<String>,
    /// 歌曲副标题
    pub subtitle: Option<String>,
    /// 艺术家名称
    pub artist: Option<String>,
    /// 歌曲副艺术家
    pub subartist: Option<String>,
    /// 文件下载链接
    pub url: Option<String>,
    /// 差分文件下载链接（可选）
    pub url_diff: Option<String>,
    /// 额外数据
    pub extra: Value,
}

impl<'de> serde::Deserialize<'de> for ChartItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 首先将整个值解析为Value
        let value: Value = Value::deserialize(deserializer)?;

        // 提取已知字段
        let level = value
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("level"))?
            .to_string();

        let md5 = value
            .get("md5")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let sha256 = value
            .get("sha256")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let subtitle = value
            .get("subtitle")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let artist = value
            .get("artist")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let subartist = value
            .get("subartist")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let url = value
            .get("url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let url_diff = value
            .get("url_diff")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 提取额外数据（除了已知字段之外的所有数据）
        let mut extra_data = value;
        if let Some(obj) = extra_data.as_object_mut() {
            // 移除已知字段，保留额外字段
            obj.remove("level");
            obj.remove("md5");
            obj.remove("sha256");
            obj.remove("title");
            obj.remove("subtitle");
            obj.remove("artist");
            obj.remove("subartist");
            obj.remove("url");
            obj.remove("url_diff");
        }

        Ok(Self {
            level,
            md5,
            sha256,
            title,
            subtitle,
            artist,
            subartist,
            url,
            url_diff,
            extra: extra_data,
        })
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

        let result: Result<CourseInfo, _> = serde_json::from_str(json_data);
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

        let result: Result<CourseInfo, _> = serde_json::from_str(json_data);
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

        let result: Result<CourseInfo, _> = serde_json::from_str(json_data);
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
