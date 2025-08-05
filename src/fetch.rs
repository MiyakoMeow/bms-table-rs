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
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// BMS表格头信息
///
/// 包含表格的基本信息和课程配置。
/// 这个结构体对应BMS表格头JSON文件的主要结构。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
}

impl<'de> serde::Deserialize<'de> for ScoreItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ScoreItemHelper {
            level: String,
            #[serde(default)]
            id: Option<u64>,
            #[serde(default)]
            md5: Option<String>,
            #[serde(default)]
            sha256: Option<String>,
            #[serde(default)]
            title: Option<String>,
            #[serde(default)]
            artist: Option<String>,
            #[serde(default)]
            url: Option<String>,
            #[serde(default)]
            url_diff: Option<String>,
        }

        let helper = ScoreItemHelper::deserialize(deserializer)?;

        // 将空字符串转换为None
        let id = helper.id;
        let md5 = helper.md5.filter(|s| !s.is_empty());
        let sha256 = helper.sha256.filter(|s| !s.is_empty());
        let title = helper.title.filter(|s| !s.is_empty());
        let artist = helper.artist.filter(|s| !s.is_empty());
        let url = helper.url.filter(|s| !s.is_empty());
        let url_diff = helper.url_diff.filter(|s| !s.is_empty());

        Ok(ScoreItem {
            level: helper.level,
            id,
            md5,
            sha256,
            title,
            artist,
            url,
            url_diff,
        })
    }
}

/// 从HTML页面中提取bmstable字段指向的JSON文件URL
pub async fn extract_bmstable_url(html_url: &str) -> Result<String> {
    let response = Client::new().get(html_url).send().await?;
    let html_content = response.text().await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试解析器创建功能
    #[tokio::test]
    async fn test_parser_creation() {
        assert!(extract_bmstable_url("https://stellabms.xyz/sl/table.html")
            .await
            .is_ok());
    }
}
