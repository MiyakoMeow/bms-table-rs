//! 反序列化实现模块
//!
//! 将所有 `Deserialize` 实现与辅助原始类型集中于此，保持 `lib.rs` 仅包含类型定义。

use anyhow::Result;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::{BmsTableData, BmsTableHeader, ChartItem, CourseInfo, Trophy};

/// 内部辅助类型：用于更简洁地反序列化表头，并保留未知字段。
#[derive(Deserialize)]
struct BmsTableHeaderRaw {
    name: String,
    symbol: String,
    data_url: String,
    #[serde(default)]
    course: Option<Value>,
    #[serde(default)]
    level_order: Option<Vec<Value>>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

/// 支持两种 `course` 输入：`Vec<CourseInfo>` 或 `Vec<Vec<CourseInfo>>`。
/// 保留原有行为：当 `course` 是空数组时，视为扁平形式并包一层为空组。
impl TryFrom<BmsTableHeaderRaw> for BmsTableHeader {
    type Error = String;

    fn try_from(raw: BmsTableHeaderRaw) -> Result<Self, Self::Error> {
        let course = match raw.course {
            Some(Value::Array(arr)) => {
                if arr.is_empty() {
                    // 空数组按扁平形式处理，包成一个空组
                    vec![Vec::new()]
                } else if matches!(arr.first(), Some(Value::Array(_))) {
                    // Vec<Vec<CourseInfo>>
                    serde_json::from_value::<Vec<Vec<CourseInfo>>>(Value::Array(arr))
                        .map_err(|e| e.to_string())?
                } else {
                    // Vec<CourseInfo> -> 包装为单个组
                    let inner: Vec<CourseInfo> =
                        serde_json::from_value(Value::Array(arr)).map_err(|e| e.to_string())?;
                    vec![inner]
                }
            }
            _ => Vec::new(),
        };

        let level_order = raw
            .level_order
            .unwrap_or_default()
            .into_iter()
            .map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s,
                other => other.to_string(),
            })
            .collect::<Vec<String>>();

        Ok(Self {
            name: raw.name,
            symbol: raw.symbol,
            data_url: raw.data_url,
            course,
            level_order,
            extra: Value::Object(raw.extra),
        })
    }
}

impl<'de> Deserialize<'de> for BmsTableHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = BmsTableHeaderRaw::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// 内部辅助类型：兼容数组或包裹对象的两种 `charts` 形式。
#[derive(Deserialize)]
#[serde(untagged)]
enum ChartsWrapper {
    Array(Vec<ChartItem>),
    Object { charts: Vec<ChartItem> },
}

impl TryFrom<ChartsWrapper> for BmsTableData {
    type Error = String;

    fn try_from(w: ChartsWrapper) -> Result<Self, Self::Error> {
        let charts = match w {
            ChartsWrapper::Array(v) => v,
            ChartsWrapper::Object { charts } => charts,
        };
        Ok(Self { charts })
    }
}

impl<'de> Deserialize<'de> for BmsTableData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let w = ChartsWrapper::deserialize(deserializer)?;
        Self::try_from(w).map_err(serde::de::Error::custom)
    }
}

/// 内部辅助类型：用于更简洁地构造 `CourseInfo`，并处理 md5/sha256 列表。
#[derive(Deserialize)]
struct CourseInfoRaw {
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

impl TryFrom<CourseInfoRaw> for CourseInfo {
    type Error = String;

    fn try_from(raw: CourseInfoRaw) -> Result<Self, Self::Error> {
        let mut charts: Vec<ChartItem> =
            Vec::with_capacity(raw.charts.len() + raw.md5list.len() + raw.sha256list.len());

        // 处理 charts，缺失 level 时补 "0"
        for mut chart_value in raw.charts {
            if chart_value.get("level").is_none() {
                let obj = chart_value
                    .as_object()
                    .ok_or_else(|| "chart_value is not an object".to_string())?
                    .clone();
                let mut obj = obj;
                obj.insert("level".to_string(), Value::String("0".to_string()));
                chart_value = Value::Object(obj);
            }
            let item: ChartItem = serde_json::from_value(chart_value).map_err(|e| e.to_string())?;
            charts.push(item);
        }

        // md5list -> charts
        charts.extend(raw.md5list.into_iter().map(|md5| ChartItem {
            level: "0".to_string(),
            md5: Some(md5),
            sha256: None,
            title: None,
            subtitle: None,
            artist: None,
            subartist: None,
            url: None,
            url_diff: None,
            extra: Value::Object(serde_json::Map::new()),
        }));

        // sha256list -> charts
        charts.extend(raw.sha256list.into_iter().map(|sha256| ChartItem {
            level: "0".to_string(),
            md5: None,
            sha256: Some(sha256),
            title: None,
            subtitle: None,
            artist: None,
            subartist: None,
            url: None,
            url_diff: None,
            extra: Value::Object(serde_json::Map::new()),
        }));

        Ok(Self {
            name: raw.name,
            constraint: raw.constraint,
            trophy: raw.trophy,
            charts,
        })
    }
}

impl<'de> Deserialize<'de> for CourseInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = CourseInfoRaw::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// 将空字符串反序列化为 `None` 的通用辅助函数。
fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}

/// 内部辅助类型：用于更简洁地反序列化 `ChartItem` 并保留未知字段。
#[derive(Deserialize)]
struct ChartItemRaw {
    level: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    md5: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    sha256: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    title: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    subtitle: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    artist: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    subartist: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    url: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    url_diff: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

impl TryFrom<ChartItemRaw> for ChartItem {
    type Error = String;

    fn try_from(raw: ChartItemRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            level: raw.level,
            md5: raw.md5,
            sha256: raw.sha256,
            title: raw.title,
            subtitle: raw.subtitle,
            artist: raw.artist,
            subartist: raw.subartist,
            url: raw.url,
            url_diff: raw.url_diff,
            extra: Value::Object(raw.extra),
        })
    }
}

impl<'de> Deserialize<'de> for ChartItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = ChartItemRaw::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}
