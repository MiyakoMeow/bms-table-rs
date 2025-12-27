//! BMS difficulty table fetching and parsing
//!
//! Provides building a complete BMS difficulty table data structure from a web page or a header JSON, covering the header, courses, trophies, and chart items.
//! Combined with feature flags, it implements network fetching and HTML parsing, suitable for CLI tools, server programs, or data-processing pipelines.
//!
//! # Feature overview
//!
//! - Parse header JSON into [`BmsTableHeader`], preserving unrecognized fields in `extra` for forward compatibility;
//! - Parse chart data into [`BmsTableData`], supporting a plain array of [`ChartItem`] structure;
//! - Courses automatically convert `md5`/`sha256` lists into chart items, filling missing `level` with "0";
//! - Extract the header JSON URL from HTML `<meta name="bmstable">`;
//! - One-stop network fetching APIs (web page → header JSON → chart data);
//! - Support fetching a list of difficulty tables into [`BmsTableList`]. [An example source page](https://darksabun.club/table/tablelist.html).
//!
//! # Feature flags
//!
//! - `serde`: enable serialization/deserialization support for types (enabled by default).
//! - `scraper`: enable HTML parsing and bmstable header URL extraction (enabled by default; implicitly enabled by `reqwest`).
//! - `reqwest`: enable the network fetching implementation (enabled by default; requires the `tokio` runtime).
//!
//! # Quick start (network fetching)
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # #[cfg(feature = "reqwest")]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::Fetcher;
//!
//! let fetcher = Fetcher::lenient()?;
//! let table = fetcher.fetch_table("https://stellabms.xyz/sl/table.html").await?;
//! println!("{}: {} charts", table.header.name, table.data.charts.len());
//! # Ok(())
//! # }
//! #
//! # #[cfg(not(feature = "reqwest"))]
//! # fn main() {}
//! ```
//!
//! # Offline usage (parse JSON directly)
//!
//! ```rust
//! # #[cfg(feature = "serde")]
//! # fn main() -> anyhow::Result<()> {
//! use bms_table::{BmsTable, BmsTableHeader, BmsTableData};
//!
//! let header_json = r#"{ "name": "Test", "symbol": "t", "data_url": "charts.json", "course": [], "level_order": [] }"#;
//! let data_json = r#"[]"#;
//! let header: BmsTableHeader = serde_json::from_str(header_json)?;
//! let data: BmsTableData = serde_json::from_str(data_json)?;
//! let _table = BmsTable { header, data };
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "serde"))]
//! # fn main() {}
//! ```
//!
//! # Example: fetch table list
//!
//! ```rust,no_run
//! # #[tokio::main]
//! # #[cfg(feature = "reqwest")]
//! # async fn main() -> anyhow::Result<()> {
//! use bms_table::fetch::reqwest::Fetcher;
//! let fetcher = Fetcher::lenient()?;
//! let listes = fetcher.fetch_table_list("https://example.com/table_list.json").await?;
//! assert!(!listes.is_empty());
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "reqwest"))]
//! # fn main() {}
//! ```
//!
//! Hint: enabling the `reqwest` feature implicitly enables `scraper` to support locating the bmstable header URL from page content.

#![warn(missing_docs)]
#![warn(clippy::must_use_candidate)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod de;
pub mod fetch;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_json::Value;
#[cfg(feature = "serde")]
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use crate::de::{de_numstring, deserialize_course_groups, deserialize_level_order};

/// Top-level BMS difficulty table data structure.
///
/// Packs header metadata and chart data together to simplify passing and use in applications.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTable {
    /// Header information and extra fields
    pub header: BmsTableHeader,
    /// Table data containing the chart list
    pub data: BmsTableData,
}

/// BMS header information.
///
/// Strictly parses common fields and preserves unrecognized fields in `extra` for forward compatibility.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTableHeader {
    /// Table name, e.g. "Satellite"
    pub name: String,
    /// Table symbol, e.g. "sl"
    pub symbol: String,
    /// URL of chart data file (preserves the original string from header JSON)
    pub data_url: String,
    /// Course information as an array of course groups
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_course_groups")
    )]
    pub course: Vec<Vec<CourseInfo>>,
    /// Difficulty level order containing numbers and strings
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_level_order")
    )]
    pub level_order: Vec<String>,
    /// Extra data (unrecognized fields from header JSON)
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: BTreeMap<String, Value>,
}

/// BMS table data.
///
/// Contains only the chart array. Parsing supports both a plain array and `{ charts: [...] }` input forms.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BmsTableData {
    /// Charts
    pub charts: Vec<ChartItem>,
}

/// Course information.
///
/// Describes a course's name, constraints, trophies and chart set. During parsing, `md5`/`sha256` lists are automatically converted into `ChartItem`s, and charts missing `level` are filled with default value `"0"`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct CourseInfo {
    /// Course name, e.g. "Satellite Skill Analyzer 2nd sl0"
    pub name: String,
    /// Constraint list, e.g. ["`grade_mirror`", "`gauge_lr2`", "ln"]
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraint: Vec<String>,
    /// List of trophies, defining requirements for different ranks
    #[cfg_attr(feature = "serde", serde(default))]
    pub trophy: Vec<Trophy>,
    /// List of charts included in the course
    #[cfg_attr(feature = "serde", serde(default))]
    pub charts: Vec<ChartItem>,
}

/// Chart data item.
///
/// Describes metadata and resource links for a single BMS file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartItem {
    /// Difficulty level, e.g. "0"
    #[cfg_attr(feature = "serde", serde(default, deserialize_with = "de_numstring"))]
    pub level: String,
    /// MD5 hash of the file
    pub md5: Option<String>,
    /// SHA256 hash of the file
    pub sha256: Option<String>,
    /// Song title
    pub title: Option<String>,
    /// Song subtitle
    pub subtitle: Option<String>,
    /// Artist name
    pub artist: Option<String>,
    /// Song sub-artist
    pub subartist: Option<String>,
    /// File download URL
    pub url: Option<String>,
    /// Differential file download URL (optional)
    pub url_diff: Option<String>,
    /// Extra data
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: BTreeMap<String, Value>,
}

/// Trophy information.
///
/// Defines conditions to achieve specific trophies, including maximum miss rate and minimum score rate.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trophy {
    /// Trophy name, e.g. "silvermedal" or "goldmedal"
    pub name: String,
    /// Maximum miss rate (percent), e.g. 5.0 means at most 5% miss rate
    pub missrate: f64,
    /// Minimum score rate (percent), e.g. 70.0 means at least 70% score rate
    pub scorerate: f64,
}

/// Complete set of original JSON strings.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTableRaw {
    /// Full URL of the header JSON
    #[cfg(feature = "scraper")]
    pub header_json_url: url::Url,
    /// Raw header JSON string
    pub header_raw: String,
    /// Full URL of the chart data JSON
    #[cfg(feature = "scraper")]
    pub data_json_url: url::Url,
    /// Raw chart data JSON string
    pub data_raw: String,
}

/// BMS difficulty table list item.
///
/// Represents the basic information of a difficulty table in a list. Only `name`, `symbol`, and `url` are required; other fields such as `tag1`, `tag2`, `comment`, `date`, `state`, and `tag_order` are collected into `extra`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmsTableInfo {
    /// Table name, e.g. ".WAS Difficulty Table"
    pub name: String,
    /// Table symbol, e.g. "．" or "[F]"
    pub symbol: String,
    /// Table URL (as a full `url::Url` type)
    #[cfg(feature = "scraper")]
    pub url: url::Url,
    /// Table URL (as a full `url::Url` type)
    #[cfg(not(feature = "scraper"))]
    pub url: String,
    /// Extra fields collection (stores all data except required fields)
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: BTreeMap<String, Value>,
}

/// Wrapper type for the list of BMS difficulty tables.
///
/// Transparently serialized as an array: serialization/deserialization behaves the same as the internal `Vec<BmsTableInfo>`, resulting in a JSON array rather than an object.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BmsTableList {
    /// List of entries
    pub listes: Vec<BmsTableInfo>,
}
