//! 示例程序
//!

#![warn(missing_docs)]
#![warn(clippy::must_use_candidate)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod fetch;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// BMS难度表数据，看这一个就够了
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTable {
    /// 表头信息与额外字段
    pub header: BmsTableHeader,
    /// 表数据，包含谱面列表
    pub data: BmsTableData,
}

/// BMS表头信息
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

/// BMS表数据
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
/// 奖杯信息
///
/// 定义了获得特定奖杯需要达到的谱面要求。
#[derive(Debug, Clone, serde::Deserialize, Serialize, PartialEq)]
pub struct Trophy {
    /// 奖杯名称，如 "silvermedal" 或 "goldmedal"
    pub name: String,
    /// 最大miss率（百分比），如 5.0 表示最大5%的miss率
    pub missrate: f64,
    /// 最小得分率（百分比），如 70.0 表示至少70%的得分率
    pub scorerate: f64,
}
