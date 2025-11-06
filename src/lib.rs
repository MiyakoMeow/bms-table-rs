//! BMS 难度表数据获取与解析库
//!
//! 提供从网页或 JSON 源构建完整的 BMS 难度表数据结构，涵盖表头、课程、奖杯与谱面条目等。
//! 结合可选特性实现网络抓取与 HTML 解析，适用于 CLI 工具、服务端程序或数据处理流水线。
//!
//! # 功能概述
//!
//! - 解析表头 JSON，支持收集未识别的额外字段
//! - 解析谱面数据，兼容数组或 `{ charts: [...] }` 两种格式
//! - 课程数据支持从 `md5`/`sha256` 列表自动转换为谱面条目
//! - 可选特性 `reqwest` 提供一站式网络获取接口
//! - 可选特性 `scraper` 支持从 HTML `<meta name="bmstable">` 提取头部 JSON 地址
//!
//! # 快速上手
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::fetch_bms_table;
//!
//! let table = fetch_bms_table("https://stellabms.xyz/sl/table.html").await?;
//! println!("{}: {} charts", table.header.name, table.data.charts.len());
//! # Ok(())
//! # }
//! ```
//!
//! # 特性说明
//!
//! - `reqwest`：启用网络获取功能（默认启用）
//! - `scraper`：启用 HTML 解析（用于从页面提取 bmstable 头部地址）

#![warn(missing_docs)]
#![warn(clippy::must_use_candidate)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod fetch;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// 顶层 BMS 难度表数据结构。
///
/// 将表头元数据与谱面数据打包在一起，便于在应用中一次性传递与使用。
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTable {
    /// 表头信息与额外字段
    pub header: BmsTableHeader,
    /// 表数据，包含谱面列表
    pub data: BmsTableData,
}

/// BMS 表头信息。
///
/// 该结构严格解析常见字段，并把未识别的字段保存在 `extra` 中，保证向前兼容。
#[derive(Debug, Clone, PartialEq, Serialize)]
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

impl<'de> serde::Deserialize<'de> for BmsTableHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 先把整个JSON作为Value读取，便于提取已知字段并收集额外字段
        let mut value: Value = Value::deserialize(deserializer)?;

        // name
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("name"))?
            .to_string();

        // symbol
        let symbol = value
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("symbol"))?
            .to_string();

        // data_url
        let data_url = value
            .get("data_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("data_url"))?
            .to_string();

        // course：支持 Vec<CourseInfo> 和 Vec<Vec<CourseInfo>> 两种格式
        let course = match value.get("course") {
            Some(Value::Array(arr)) if !arr.is_empty() => {
                if matches!(arr.first(), Some(Value::Array(_))) {
                    // Vec<Vec<CourseInfo>>
                    serde_json::from_value::<Vec<Vec<CourseInfo>>>(value["course"].clone())
                        .map_err(serde::de::Error::custom)?
                } else {
                    // Vec<CourseInfo> -> 包装为 Vec<Vec<CourseInfo>>
                    let inner: Vec<CourseInfo> = serde_json::from_value(value["course"].clone())
                        .map_err(serde::de::Error::custom)?;
                    vec![inner]
                }
            }
            _ => Vec::new(),
        };

        // level_order：数字和字符串统一转为字符串
        let level_order = match value.get("level_order") {
            Some(Value::Array(arr)) => arr
                .iter()
                .map(|v| match v {
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect::<Vec<String>>(),
            _ => Vec::new(),
        };

        // 收集额外字段：移除已知字段后剩余的内容
        if let Some(obj) = value.as_object_mut() {
            obj.remove("name");
            obj.remove("symbol");
            obj.remove("data_url");
            obj.remove("course");
            obj.remove("level_order");
        }

        Ok(Self {
            name,
            symbol,
            data_url,
            course,
            level_order,
            extra: value,
        })
    }
}

/// BMS 表数据。
///
/// 仅包含谱面数组。解析时同时兼容纯数组与 `{ charts: [...] }` 两种输入形式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BmsTableData {
    /// 谱面数据
    pub charts: Vec<ChartItem>,
}

impl<'de> serde::Deserialize<'de> for BmsTableData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 支持两种输入：
        // 1) 直接是数组： [ {...}, {...} ]
        // 2) 对象包裹： { "charts": [ {...}, {...} ] }
        let value: Value = Value::deserialize(deserializer)?;
        match value {
            Value::Array(arr) => {
                let charts: Vec<ChartItem> =
                    serde_json::from_value(Value::Array(arr)).map_err(serde::de::Error::custom)?;
                Ok(Self { charts })
            }
            Value::Object(mut obj) => {
                let charts_value = obj.remove("charts").unwrap_or(Value::Array(vec![]));
                let charts: Vec<ChartItem> =
                    serde_json::from_value(charts_value).map_err(serde::de::Error::custom)?;
                Ok(Self { charts })
            }
            _ => Err(serde::de::Error::custom(
                "BmsTableData expects array or object with charts",
            )),
        }
    }
}

/// 课程信息。
///
/// 描述一个课程的名称、约束、奖杯与谱面集合。解析阶段会将 `md5`/`sha256`
/// 列表自动转换为对应的 `ChartItem`，并为缺失 `level` 的谱面补充默认值 `"0"`。
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

/// 谱面数据项。
///
/// 描述单个 BMS 文件的相关元数据与资源链接。为空字符串的可选字段在反序列化时会
/// 自动转换为 `None`，以提升数据质量。
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
/// 奖杯信息。
///
/// 定义达成特定奖杯的条件，包括最大 miss 率与最低得分率等要求。
#[derive(Debug, Clone, serde::Deserialize, Serialize, PartialEq)]
pub struct Trophy {
    /// 奖杯名称，如 "silvermedal" 或 "goldmedal"
    pub name: String,
    /// 最大miss率（百分比），如 5.0 表示最大5%的miss率
    pub missrate: f64,
    /// 最小得分率（百分比），如 70.0 表示至少70%的得分率
    pub scorerate: f64,
}
