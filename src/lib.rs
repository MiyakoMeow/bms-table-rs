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
//! # #[cfg(feature = "reqwest")]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::fetch_bms_table;
//!
//! let table = fetch_bms_table("https://stellabms.xyz/sl/table.html").await?;
//! println!("{}: {} charts", table.header.name, table.data.charts.len());
//! # Ok(())
//! # }
//! #
//! # #[cfg(not(feature = "reqwest"))]
//! # fn main() {}
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

pub mod de;
pub mod fetch;
pub mod ser;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_json::Value;
use std::collections::HashMap;

/// 顶层 BMS 难度表数据结构。
///
/// 将表头元数据与谱面数据打包在一起，便于在应用中一次性传递与使用。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTable {
    /// 表头信息与额外字段
    pub header: BmsTableHeader,
    /// 表数据，包含谱面列表
    pub data: BmsTableData,
}

/// BMS 表头信息。
///
/// 该结构严格解析常见字段，并把未识别的字段保存在 `extra` 中，保证向前兼容。
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
    #[cfg(feature = "serde")]
    pub extra: HashMap<String, Value>,
}

/// BMS 表数据。
///
/// 仅包含谱面数组。解析时同时兼容纯数组与 `{ charts: [...] }` 两种输入形式。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BmsTableData {
    /// 谱面数据
    pub charts: Vec<ChartItem>,
}

/// 课程信息。
///
/// 描述一个课程的名称、约束、奖杯与谱面集合。解析阶段会将 `md5`/`sha256`
/// 列表自动转换为对应的 `ChartItem`，并为缺失 `level` 的谱面补充默认值 `"0"`。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CourseInfo {
    /// 课程名称，如 "Satellite Skill Analyzer 2nd sl0"
    pub name: String,
    /// 约束条件列表，如 ["grade_mirror", "gauge_lr2", "ln"]
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraint: Vec<String>,
    /// 奖杯信息列表，定义不同等级的奖杯要求
    #[cfg_attr(feature = "serde", serde(default))]
    pub trophy: Vec<Trophy>,
    /// 谱面数据列表，包含该课程的所有谱面信息
    #[cfg_attr(feature = "serde", serde(default))]
    pub charts: Vec<ChartItem>,
}

/// 谱面数据项。
///
/// 描述单个 BMS 文件的相关元数据与资源链接。为空字符串的可选字段在反序列化时会
/// 自动转换为 `None`，以提升数据质量。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartItem {
    /// 难度等级，如 "0"
    pub level: String,
    /// 文件的MD5哈希值
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub md5: Option<String>,
    /// 文件的SHA256哈希值
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub sha256: Option<String>,
    /// 歌曲标题
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub title: Option<String>,
    /// 歌曲副标题
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub subtitle: Option<String>,
    /// 艺术家名称
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub artist: Option<String>,
    /// 歌曲副艺术家
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub subartist: Option<String>,
    /// 文件下载链接
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub url: Option<String>,
    /// 差分文件下载链接（可选）
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "crate::de::empty_string_as_none")
    )]
    pub url_diff: Option<String>,
    /// 额外数据
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: HashMap<String, Value>,
}

/// 奖杯信息。
///
/// 定义达成特定奖杯的条件，包括最大 miss 率与最低得分率等要求。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trophy {
    /// 奖杯名称，如 "silvermedal" 或 "goldmedal"
    pub name: String,
    /// 最大miss率（百分比），如 5.0 表示最大5%的miss率
    pub missrate: f64,
    /// 最小得分率（百分比），如 70.0 表示至少70%的得分率
    pub scorerate: f64,
}

/// 完整的原始 JSON 字符串集合。
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTableRaw {
    /// 原始表头 JSON 字符串
    pub header_raw: String,
    /// 原始谱面数据 JSON 字符串
    pub data_raw: String,
}

/// BMS 表索引条目。
///
/// 表示一个难度表在索引列表中的基本信息。仅要求 `name`、`symbol`、`url` 三个字段，
/// 其余诸如 `tag1`、`tag2`、`comment`、`date`、`state`、`tag_order` 等字段统一收集到 `extra`。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTableIndexItem {
    /// 表名称，如 ".WAS難易度表"
    pub name: String,
    /// 表符号，如 "．" 或 "[F]"
    pub symbol: String,
    /// 表地址（为完整的 `url::Url` 类型）
    pub url: url::Url,
    /// 额外字段集合（用于保存除必需字段外的所有数据）
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: HashMap<String, Value>,
}

/// BMS 表索引列表包装类型。
///
/// 透明序列化为数组：序列化/反序列化时行为与内部的 `Vec<BmsTableIndexItem>` 相同，
/// 因此序列化结果为一个 JSON 数组而不是对象。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BmsTableIndex {
    /// 索引条目数组
    pub indexes: Vec<BmsTableIndexItem>,
}
