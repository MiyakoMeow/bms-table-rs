//! BMS表格数据获取模块
//!
//! 这个模块提供了从BMS表格网站获取和解析数据的功能。
//! 支持从HTML页面提取bmstable字段，解析JSON格式的表格头信息和分数数据。
//!
//! # 主要功能
//!
//! - 从HTML页面中提取bmstable字段指向的JSON文件URL
//! - 解析BMS表格头信息（包含课程、奖杯等元数据）
//! - 获取和解析分数数据（包含歌曲信息、下载链接等）
//! - 完整的BMS表格数据获取流程

use anyhow::Result;
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
    /// 分数数据文件的相对URL，如 "score.json"
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
            course: Vec<Vec<CourseInfo>>,
            #[serde(default)]
            level_order: Option<Vec<Value>>,
        }

        let helper = BmsTableHeaderHelper::deserialize(deserializer)?;

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

        Ok(BmsTableHeader {
            name: helper.name,
            symbol: helper.symbol,
            data_url: helper.data_url,
            course: helper.course,
            level_order,
        })
    }
}

/// 课程信息
///
/// 定义了一个BMS课程的所有相关信息，包括约束条件、奖杯要求和MD5哈希列表。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CourseInfo {
    /// 课程名称，如 "Satellite Skill Analyzer 2nd sl0"
    pub name: String,
    /// 约束条件列表，如 ["grade_mirror", "gauge_lr2", "ln"]
    #[serde(default)]
    pub constraint: Vec<String>,
    /// 奖杯信息列表，定义不同等级的奖杯要求
    #[serde(default)]
    pub trophy: Vec<Trophy>,
    /// 该课程包含的BMS文件的MD5哈希列表
    pub md5: Vec<String>,
}

/// 奖杯信息
///
/// 定义了获得特定奖杯需要达到的分数要求。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Trophy {
    /// 奖杯名称，如 "silvermedal" 或 "goldmedal"
    pub name: String,
    /// 最大miss率（百分比），如 5.0 表示最大5%的miss率
    pub missrate: f64,
    /// 最小得分率（百分比），如 70.0 表示至少70%的得分率
    pub scorerate: f64,
}

/// 分数数据项
///
/// 表示一个BMS文件的分数数据，包含文件信息和下载链接。
/// 所有字段都是可选的，因为不同的BMS表格可能有不同的字段。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ScoreItem {
    /// 难度等级，如 "0"
    pub level: String,
    /// 唯一标识符
    pub id: Option<u64>,
    /// 文件的MD5哈希值
    pub md5: Option<String>,
    /// 文件的SHA256哈希值
    pub sha256: Option<String>,
    /// 歌曲标题
    pub title: Option<String>,
    /// 艺术家名称
    pub artist: Option<String>,
    /// 文件下载链接
    pub url: Option<String>,
    /// 差分文件下载链接（可选）
    pub url_diff: Option<String>,
    /// 额外数据
    pub extra: Value,
}

impl<'de> serde::Deserialize<'de> for ScoreItem {
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

        let id = value.get("id").and_then(|v| v.as_u64());
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
        let artist = value
            .get("artist")
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
        let mut extra_data = value.clone();
        if let Some(obj) = extra_data.as_object_mut() {
            // 移除已知字段，保留额外字段
            obj.remove("level");
            obj.remove("id");
            obj.remove("md5");
            obj.remove("sha256");
            obj.remove("title");
            obj.remove("artist");
            obj.remove("url");
            obj.remove("url_diff");
        }

        Ok(ScoreItem {
            level,
            id,
            md5,
            sha256,
            title,
            artist,
            url,
            url_diff,
            extra: extra_data,
        })
    }
}

/// 从HTML页面内容中，提取bmstable字段指向的JSON文件URL
pub async fn extract_bmstable_url(html_content: &str) -> Result<String> {
    let document = Html::parse_document(&html_content);

    // 查找所有meta标签
    let meta_selector = Selector::parse("meta").unwrap();

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

    Err(anyhow::anyhow!("未找到bmstable字段"))
}

/// 判断内容是否为JSON格式
#[allow(dead_code)]
pub(super) fn is_json_content(content: &str) -> bool {
    content.trim().starts_with('{') || content.trim().starts_with('[')
}

/// 从header JSON和base URL获取data JSON
///
/// # 参数
///
/// * `header_json` - header的JSON解析树
/// * `base_url` - 基础URL，用于构建data文件的完整URL
///
/// # 返回值
///
/// 返回data的JSON解析树
///
/// # 错误
///
/// 如果无法获取data文件或解析失败，将返回错误
#[allow(dead_code)]
pub(super) async fn fetch_data_json(header_json: &Value, base_url: &str) -> Result<Value> {
    let header: BmsTableHeader = serde_json::from_value(header_json.clone())?;
    let base_url_obj = Url::parse(base_url)?;
    let data_url = base_url_obj.join(&header.data_url)?;
    let data_url_str = data_url.as_str();

    let data_response = reqwest::Client::new().get(data_url_str).send().await?;
    let data_json_content = data_response.text().await?;
    let data_json: Value = serde_json::from_str(&data_json_content)?;

    Ok(data_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试解析器创建功能
    #[tokio::test]
    async fn test_parser_creation() {
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

        let result = extract_bmstable_url(html_content).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "header.json");
    }

    /// 测试解析器处理没有bmstable字段的HTML
    #[tokio::test]
    async fn test_parser_no_bmstable() {
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

        let result = extract_bmstable_url(html_content).await;
        assert!(result.is_err());
    }
}
