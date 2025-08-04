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
//! 
//! # 使用示例
//! 
//! ```rust
//! use bms_table::fetch::BmsTableParser;
//! use anyhow::Result;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let parser = BmsTableParser::new();
//!     
//!     // 注意：这个示例需要可访问的BMS表格网站
//!     // let (header, scores) = parser.fetch_complete_table("https://example.com/table.html").await?;
//!     // println!("表格名称: {}", header.name);
//!     // println!("分数数据数量: {}", scores.len());
//!     
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

/// BMS表格头信息
/// 
/// 包含表格的基本信息和课程配置。
/// 这个结构体对应BMS表格头JSON文件的主要结构。
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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

/// BMS表格解析器
/// 
/// 提供从BMS表格网站获取和解析数据的功能。
/// 使用HTTP客户端来获取HTML和JSON数据，并提供完整的解析流程。
pub struct BmsTableParser {
    /// HTTP客户端，用于发送请求
    client: Client,
}

impl BmsTableParser {
    /// 创建新的BMS表格解析器实例
    /// 
    /// # 返回值
    /// 
    /// 返回一个配置好的解析器实例，包含HTTP客户端。
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// use bms_table::fetch::BmsTableParser;
    /// 
    /// let parser = BmsTableParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 从HTML页面中提取bmstable字段
    /// 
    /// 解析HTML页面的head标签，查找包含bmstable字段的meta标签，
    /// 提取指向JSON配置文件的URL。
    /// 
    /// # 参数
    /// 
    /// * `html_url` - HTML页面的URL
    /// 
    /// # 返回值
    /// 
    /// 返回提取到的bmstable URL字符串，如果未找到则返回错误。
    /// 
    /// # 错误
    /// 
    /// 如果无法获取HTML页面、解析失败或未找到bmstable字段，将返回错误。
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use bms_table::fetch::BmsTableParser;
    /// 
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let parser = BmsTableParser::new();
    ///     let url = parser.extract_bmstable_url("https://example.com/table.html").await?;
    ///     println!("bmstable URL: {}", url);
    ///     Ok(())
    /// }
    /// ```
    pub async fn extract_bmstable_url(&self, html_url: &str) -> Result<String> {
        let response = self.client.get(html_url).send().await?;
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

    /// 获取并解析BMS表格头信息
    /// 
    /// 从指定的URL获取JSON格式的BMS表格头信息，并解析为结构体。
    /// 
    /// # 参数
    /// 
    /// * `header_url` - 表格头信息JSON文件的URL
    /// 
    /// # 返回值
    /// 
    /// 返回解析后的BmsTableHeader结构体，包含表格名称、符号、课程信息等。
    /// 
    /// # 错误
    /// 
    /// 如果无法获取JSON文件或解析失败，将返回错误。
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use bms_table::fetch::BmsTableParser;
    /// 
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let parser = BmsTableParser::new();
    ///     let header = parser.get_table_header("https://example.com/header.json").await?;
    ///     println!("表格名称: {}", header.name);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_table_header(&self, header_url: &str) -> Result<BmsTableHeader> {
        let response = self.client.get(header_url).send().await?;
        let json_content = response.text().await?;

        let header: BmsTableHeader = serde_json::from_str(&json_content)?;
        Ok(header)
    }

    /// 获取并解析分数数据
    /// 
    /// 从指定的URL获取JSON格式的分数数据，并解析为结构体数组。
    /// 
    /// # 参数
    /// 
    /// * `score_url` - 分数数据JSON文件的URL
    /// 
    /// # 返回值
    /// 
    /// 返回解析后的ScoreItem数组，包含所有BMS文件的分数数据。
    /// 
    /// # 错误
    /// 
    /// 如果无法获取JSON文件或解析失败，将返回错误。
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use bms_table::fetch::BmsTableParser;
    /// 
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let parser = BmsTableParser::new();
    ///     let scores = parser.get_score_data("https://example.com/score.json").await?;
    ///     println!("分数数据数量: {}", scores.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_score_data(&self, score_url: &str) -> Result<Vec<ScoreItem>> {
        let response = self.client.get(score_url).send().await?;
        let json_content = response.text().await?;

        let scores: Vec<ScoreItem> = serde_json::from_str(&json_content)?;
        Ok(scores)
    }

    /// 完整的BMS表格数据获取流程
    /// 
    /// 执行完整的BMS表格数据获取流程：
    /// 1. 从HTML页面提取bmstable字段
    /// 2. 获取并解析表格头信息
    /// 3. 获取并解析分数数据
    /// 
    /// # 参数
    /// 
    /// * `base_url` - BMS表格HTML页面的URL
    /// 
    /// # 返回值
    /// 
    /// 返回一个元组，包含表格头信息和分数数据数组。
    /// 
    /// # 错误
    /// 
    /// 如果在任何步骤中发生错误（网络错误、解析错误等），将返回错误。
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// use bms_table::fetch::BmsTableParser;
    /// 
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let parser = BmsTableParser::new();
    ///     let (header, scores) = parser.fetch_complete_table("https://example.com/table.html").await?;
    ///     
    ///     println!("表格名称: {}", header.name);
    ///     println!("分数数据数量: {}", scores.len());
    ///     
    ///     // 显示第一个分数数据
    ///     if let Some(first_score) = scores.first() {
    ///         if let Some(title) = &first_score.title {
    ///             println!("第一个歌曲: {}", title);
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn fetch_complete_table(
        &self,
        base_url: &str,
    ) -> Result<(BmsTableHeader, Vec<ScoreItem>)> {
        // 1. 从HTML页面提取bmstable URL
        let bmstable_url = self.extract_bmstable_url(base_url).await?;

        // 2. 解析bmstable URL为绝对路径
        let base_url_obj = Url::parse(base_url)?;
        let header_url = base_url_obj.join(&bmstable_url)?;

        // 3. 获取表格头信息
        let header = self.get_table_header(header_url.as_str()).await?;

        // 4. 构建分数数据URL
        let score_url = header_url.join(&header.data_url)?;

        // 5. 获取分数数据
        let scores = self.get_score_data(score_url.as_str()).await?;

        Ok((header, scores))
    }
}

impl Default for BmsTableParser {
    /// 创建默认的BMS表格解析器实例
    /// 
    /// 等同于调用 `BmsTableParser::new()`。
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试解析器创建功能
    #[tokio::test]
    async fn test_parser_creation() {
        let parser = BmsTableParser::new();
        assert!(parser
            .client
            .get("https://stellabms.xyz/sl/table.html")
            .send()
            .await
            .is_ok());
    }
}