//! 示例程序
//!

#![warn(missing_docs)]

mod fetch;

use anyhow::{anyhow, Result};
use serde_json::Value;
use url::Url;

use fetch::{extract_bmstable_url, ChartItem, CourseInfo};

/// BMS难度表数据，看这一个就够了
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTable {
    /// 表头信息与额外字段
    pub header: BmsTableHeader,
    /// 表数据，包含谱面列表
    pub data: BmsTableData,
}

/// BMS表头信息
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTableHeader {
    /// 表格名称，如 "Satellite"
    pub name: String,
    /// 表格符号，如 "sl"
    pub symbol: String,
    /// 谱面数据文件的URL（原样保存来自header JSON的字符串）
    pub data_url: String,
    /// 课程信息数组，每个元素是一个课程组的数组
    pub course: Vec<Vec<CourseInfo>>,
    /// 难度等级顺序，包含数字和字符串
    pub level_order: Vec<String>,
    /// 额外数据（来自header JSON中未识别的字段）
    pub extra: serde_json::Value,
}

/// BMS表数据
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BmsTableData {
    /// 谱面数据
    pub charts: Vec<ChartItem>,
}

/// 从URL直接获取BmsTable对象
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
///     println!("表格名称: {}", bms_table.header.name);
///     println!("谱面数据数量: {}", bms_table.data.charts.len());
///     Ok(())
/// }
/// ```
#[cfg(feature = "reqwest")]
pub async fn fetch_bms_table(web_url: &str) -> Result<BmsTable> {
    let web_url = Url::parse(web_url)?;
    let web_response = reqwest::Client::new()
        .get(web_url.clone())
        .send()
        .await
        .map_err(|e| anyhow!("When fetching web: {e}"))?
        .text()
        .await
        .map_err(|e| anyhow!("When parsing web response: {e}"))?;
    let (header_url, header_json) = match get_web_header_json_value(&web_response)? {
        HeaderQueryContent::Url(header_url_string) => {
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
            let HeaderQueryContent::Json(header_json) =
                get_web_header_json_value(&header_response_string)?
            else {
                return Err(anyhow!(
                    "Cycled header found. web_url: {web_url}, header_url: {header_url_string}"
                ));
            };
            (header_url, header_json)
        }
        HeaderQueryContent::Json(value) => (web_url, value),
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
    create_bms_table_from_json(header_url.as_str(), header_json, data_json)
}

/// [`get_web_header_json_value`]的返回类型
pub enum HeaderQueryContent {
    /// 注意：可能解析出相对或绝对Url，建议使用[`Url::join`]。
    Url(String),
    /// Json树
    Json(Value),
}

/// 从相应数据中提取Json树（Json内容）或Header地址（HTML内容）
pub fn get_web_header_json_value(response_str: &str) -> anyhow::Result<HeaderQueryContent> {
    use crate::fetch::is_json_content;
    // 判断返回的内容是HTML还是JSON
    if is_json_content(response_str) {
        // 如果是JSON，直接当作header处理
        let header_json: Value = serde_json::from_str(response_str)
            .map_err(|e| anyhow!("When parsing header json, Error: {e}"))?;
        Ok(HeaderQueryContent::Json(header_json))
    } else {
        let bmstable_url = extract_bmstable_url(response_str)?;
        Ok(HeaderQueryContent::Url(bmstable_url))
    }
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
/// #[cfg(feature = "reqwest")]
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
///     let bms_table = create_bms_table_from_json(header_url, header_json, data_json)?;
///     println!("表格名称: {}", bms_table.header.name);
///     Ok(())
/// }
///
/// #[cfg(not(feature = "reqwest"))]
/// fn main() {}
/// ```
pub fn create_bms_table_from_json(
    header_url: &str,
    header_json: Value,
    data_json: Value,
) -> Result<BmsTable> {
    // 解析header JSON，保留额外数据
    let raw_header: crate::fetch::BmsTableHeader = serde_json::from_value(header_json.clone())?;

    // 提取额外数据（header_json中除了BmsTableHeader字段之外的数据）
    let mut extra_data = header_json;
    if let Some(obj) = extra_data.as_object_mut() {
        // 移除已知字段，保留额外字段
        obj.remove("name");
        obj.remove("symbol");
        obj.remove("data_url");
        obj.remove("course");
        obj.remove("level_order");
    }

    // 解析data JSON
    let charts: Vec<ChartItem> = serde_json::from_value(data_json)?;

    // 解析并校验 header_url，但不在结构体中保存
    let _ = Url::parse(header_url)?;

    // 创建BmsTable对象
    let header = BmsTableHeader {
        name: raw_header.name,
        symbol: raw_header.symbol,
        data_url: raw_header.data_url,
        course: raw_header.course,
        level_order: raw_header.level_order,
        extra: extra_data,
    };

    let data = BmsTableData { charts };

    let bms_table = BmsTable { header, data };

    Ok(bms_table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::is_json_content;
    use serde_json::json;
    use url::Url;

    /// 测试创建BmsTable对象
    #[test]
    fn test_create_bms_table_from_json() {
        let header_url = "https://example.com/header.json";
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

        let result = create_bms_table_from_json(header_url, header_json, data_json);
        assert!(result.is_ok());

        let bms_table = result.unwrap();
        assert_eq!(bms_table.header.name, "Test Table");
        assert_eq!(bms_table.header.symbol, "test");
        assert_eq!(bms_table.header.data_url, "charts.json");
        assert_eq!(bms_table.header.course.len(), 1);
        assert_eq!(bms_table.data.charts.len(), 1);

        // 测试课程信息
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

        // 测试谱面数据
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

        // 测试额外数据
        // 检查header的额外数据
        assert_eq!(bms_table.header.extra["extra_field"], "extra_value");
        assert_eq!(bms_table.header.extra["another_field"], 123);
        assert!(bms_table.header.extra.get("name").is_none()); // 确保已知字段被移除

        // 检查score的额外数据
        assert_eq!(score.extra["custom_field"], "custom_value");
        assert_eq!(score.extra["rating"], 5.0);
        assert!(score.extra.get("level").is_none()); // 确保已知字段被移除

        // 测试level_order
        assert_eq!(bms_table.header.level_order.len(), 22);
        assert_eq!(bms_table.header.level_order[0], "0");
        assert_eq!(bms_table.header.level_order[20], "20");
        assert_eq!(bms_table.header.level_order[21], "!i");
        assert!(bms_table.header.extra.get("level_order").is_none()); // 确保level_order被移除
    }

    /// 测试创建BmsTable对象时处理空字符串字段
    #[test]
    fn test_create_bms_table_with_empty_fields() {
        let header_url = "https://example.com/header.json";
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

        let result = create_bms_table_from_json(header_url, header_json, data_json);
        assert!(result.is_ok());

        let bms_table = result.unwrap();
        let score = &bms_table.data.charts[0];
        assert_eq!(score.level, "1");
        assert_eq!(score.md5, None);
        assert_eq!(score.sha256, None);
        assert_eq!(score.title, None);
        assert_eq!(score.artist, None);
        assert_eq!(score.url, None);
        assert_eq!(score.url_diff, None);
    }

    /// 测试BmsTable结构体的基本功能
    #[test]
    fn test_bms_table_creation() {
        let header = BmsTableHeader {
            name: "Test Table".to_string(),
            symbol: "test".to_string(),
            data_url: "https://example.com/charts.json".to_string(),
            course: vec![],
            level_order: vec!["0".to_string(), "1".to_string()],
            extra: json!({}),
        };
        let data = BmsTableData { charts: vec![] };
        let bms_table = BmsTable { header, data };

        assert_eq!(bms_table.header.name, "Test Table");
        assert_eq!(bms_table.header.symbol, "test");
        assert_eq!(bms_table.header.course.len(), 0);
        assert_eq!(bms_table.data.charts.len(), 0);
        assert_eq!(bms_table.header.level_order.len(), 2);
    }

    /// 测试BmsTable的PartialEq实现
    #[test]
    fn test_bms_table_partial_eq() {
        let header1 = BmsTableHeader {
            name: "Test Table".to_string(),
            symbol: "test".to_string(),
            data_url: "https://example.com/charts.json".to_string(),
            course: vec![],
            level_order: vec!["0".to_string(), "1".to_string()],
            extra: json!({}),
        };
        let data1 = BmsTableData { charts: vec![] };
        let table1 = BmsTable {
            header: header1.clone(),
            data: data1,
        };

        let header2 = header1.clone();
        let data2 = BmsTableData { charts: vec![] };
        let table2 = BmsTable {
            header: header2,
            data: data2,
        };

        assert_eq!(table1, table2);
    }

    /// 测试错误处理 - 无效的JSON
    #[test]
    fn test_create_bms_table_invalid_json() {
        let header_url = "https://example.com/header.json";
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

        let result = create_bms_table_from_json(header_url, header_json, data_json);
        assert!(result.is_ok()); // 这个测试应该通过，因为缺少的字段有默认值
    }

    /// 测试错误处理 - 无效的URL
    #[test]
    fn test_create_bms_table_invalid_url() {
        let header_url = "invalid-url";
        let header_json = json!({
            "name": "Test Table",
            "symbol": "test",
            "data_url": "charts.json",
            "course": []
        });
        let data_json = json!([]);

        let result = create_bms_table_from_json(header_url, header_json, data_json);
        assert!(result.is_err());
    }

    /// 测试fetch_table_json_data函数的错误处理
    #[tokio::test]
    #[cfg(feature = "reqwest")]
    async fn test_fetch_table_json_data_invalid_url() {
        let result = fetch_bms_table("https://invalid-url-that-does-not-exist.com").await;
        assert!(result.is_err());
    }

    /// 测试fetch_bms_table函数的错误处理
    #[tokio::test]
    #[cfg(feature = "reqwest")]
    async fn test_fetch_bms_table_invalid_url() {
        let result = fetch_bms_table("https://invalid-url-that-does-not-exist.com").await;
        assert!(result.is_err());
    }

    /// 测试URL解析功能
    #[test]
    fn test_url_parsing() {
        let base_url = "https://example.com/table.html";
        let bmstable_url = "header.json";

        let base_url_obj = Url::parse(base_url).unwrap();
        let header_url = base_url_obj.join(bmstable_url).unwrap();

        assert_eq!(header_url.as_str(), "https://example.com/header.json");
    }

    /// 测试JSON序列化和反序列化
    #[test]
    fn test_json_serialization() {
        // 测试fetch模块中的BmsTableHeader序列化/反序列化
        let header = crate::fetch::BmsTableHeader {
            name: "Test Table".to_string(),
            symbol: "test".to_string(),
            data_url: "charts.json".to_string(),
            course: vec![],
            level_order: vec!["0".to_string(), "1".to_string(), "!i".to_string()],
        };

        let json = serde_json::to_string(&header).unwrap();
        let parsed: crate::fetch::BmsTableHeader = serde_json::from_str(&json).unwrap();

        assert_eq!(header, parsed);
    }

    /// 测试JSON内容判断功能
    #[test]
    fn test_is_json_content() {
        // 测试JSON对象
        assert!(is_json_content(r#"{"name": "test"}"#));
        assert!(is_json_content(r#"  {"name": "test"}  "#));

        // 测试JSON数组
        assert!(is_json_content(r#"[1, 2, 3]"#));
        assert!(is_json_content(r#"  [1, 2, 3]  "#));

        // 测试非JSON内容
        assert!(!is_json_content("<html><body>test</body></html>"));
        assert!(!is_json_content("This is plain text"));
        assert!(!is_json_content(""));
        assert!(!is_json_content("   "));
    }
}
