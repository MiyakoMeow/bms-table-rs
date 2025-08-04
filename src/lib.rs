//! 示例程序
//!

#![warn(missing_docs)]

use anyhow::Result;
use reqwest;
use serde_json::Value;
use url::Url;

use crate::fetch::{BmsTableHeader, BmsTableParser, CourseInfo, ScoreItem};

pub mod fetch;

/// BMS难度表数据，看这一个就够了
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTable {
    /// 表格名称，如 "Satellite"
    pub name: String,
    /// 表格符号，如 "sl"
    pub symbol: String,
    /// 表格头文件的相对URL，如 "header.json"
    pub header_url: Url,
    /// 分数数据文件的相对URL，如 "score.json"
    pub data_url: Url,
    /// 课程信息数组，每个元素是一个课程组的数组
    pub course: Vec<Vec<CourseInfo>>,
    /// 分数数据
    pub scores: Vec<ScoreItem>,
}

/// 从header的绝对URL地址、header和data的JSON解析树创建BmsTable对象
///
/// # 参数
///
/// * `header_url` - header文件的绝对URL地址
/// * `header_json` - header的JSON解析树
/// * `data_json` - data的JSON解析树
///
/// # 返回值
///
/// 返回解析后的BmsTable对象
///
/// # 错误
///
/// 如果JSON解析失败或URL解析失败，将返回错误
///
/// # 示例
///
/// ```rust,no_run
/// use bms_table::{create_bms_table_from_json, BmsTable};
/// use serde_json::json;
/// use url::Url;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let header_url = "https://example.com/header.json";
///     let header_json = json!({
///         "name": "Test Table",
///         "symbol": "test",
///         "data_url": "score.json",
///         "course": []
///     });
///     let data_json = json!([]);
///     
///     let bms_table = create_bms_table_from_json(header_url, header_json, data_json).await?;
///     println!("表格名称: {}", bms_table.name);
///     Ok(())
/// }
/// ```
pub async fn create_bms_table_from_json(
    header_url: &str,
    header_json: Value,
    data_json: Value,
) -> Result<BmsTable> {
    // 解析header JSON
    let header: BmsTableHeader = serde_json::from_value(header_json)?;

    // 解析data JSON
    let scores: Vec<ScoreItem> = serde_json::from_value(data_json)?;

    // 构建URL对象
    let header_url_obj = Url::parse(header_url)?;
    let data_url_obj = header_url_obj.join(&header.data_url)?;

    // 创建BmsTable对象
    let bms_table = BmsTable {
        name: header.name,
        symbol: header.symbol,
        header_url: header_url_obj,
        data_url: data_url_obj,
        course: header.course,
        scores,
    };

    Ok(bms_table)
}

/// 从URL获取header的绝对URL地址、header和data的JSON解析树
///
/// # 参数
///
/// * `url` - BMS表格HTML页面的URL
///
/// # 返回值
///
/// 返回一个元组，包含header的绝对URL地址、header的JSON解析树和data的JSON解析树
///
/// # 错误
///
/// 如果无法获取数据或解析失败，将返回错误
///
/// # 示例
///
/// ```rust,no_run
/// use bms_table::fetch_table_json_data;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (header_url, header_json, data_json) = fetch_table_json_data("https://example.com/table.html").await?;
///     println!("Header URL: {}", header_url);
///     println!("Header JSON: {:?}", header_json);
///     println!("Data JSON: {:?}", data_json);
///     Ok(())
/// }
/// ```
pub async fn fetch_table_json_data(url: &str) -> Result<(String, Value, Value)> {
    let parser = BmsTableParser::new();

    // 1. 从HTML页面提取bmstable URL
    let bmstable_url = parser.extract_bmstable_url(url).await?;

    // 2. 解析bmstable URL为绝对路径
    let base_url_obj = Url::parse(url)?;
    let header_url = base_url_obj.join(&bmstable_url)?;
    let header_url_str = header_url.as_str().to_string();

    // 3. 获取header JSON
    let header_response = reqwest::Client::new().get(&header_url_str).send().await?;
    let header_json_content = header_response.text().await?;
    let header_json: Value = serde_json::from_str(&header_json_content)?;

    // 4. 从header中提取data_url并构建data URL
    let header: BmsTableHeader = serde_json::from_value(header_json.clone())?;
    let data_url = header_url.join(&header.data_url)?;
    let data_url_str = data_url.as_str();

    // 5. 获取data JSON
    let data_response = reqwest::Client::new().get(data_url_str).send().await?;
    let data_json_content = data_response.text().await?;
    let data_json: Value = serde_json::from_str(&data_json_content)?;

    Ok((header_url_str, header_json, data_json))
}

/// 从URL直接获取BmsTable对象（合并上述两个步骤）
///
/// # 参数
///
/// * `url` - BMS表格HTML页面的URL
///
/// # 返回值
///
/// 返回解析后的BmsTable对象
///
/// # 错误
///
/// 如果无法获取数据或解析失败，将返回错误
///
/// # 示例
///
/// ```rust,no_run
/// use bms_table::fetch_bms_table;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let bms_table = fetch_bms_table("https://example.com/table.html").await?;
///     println!("表格名称: {}", bms_table.name);
///     println!("分数数据数量: {}", bms_table.scores.len());
///     Ok(())
/// }
/// ```
pub async fn fetch_bms_table(url: &str) -> Result<BmsTable> {
    let (header_url, header_json, data_json) = fetch_table_json_data(url).await?;
    create_bms_table_from_json(&header_url, header_json, data_json).await
}
