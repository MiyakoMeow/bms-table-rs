//! Deserialization implementation module
//!
//! Centralizes all `Deserialize` implementations and helper raw types here, keeping `lib.rs` focused on type definitions.
#![cfg(feature = "serde")]

use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{ChartItem, CourseInfo, Trophy};

/// Field-level deserialization: supports `course` being `Vec<CourseInfo>` or `Vec<Vec<CourseInfo>>`,
/// and returns `vec![Vec::new()]` for an empty array to preserve previous behavior.
pub(crate) fn deserialize_course_groups<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<CourseInfo>>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(Value::Array(arr)) = Option::<Value>::deserialize(deserializer)? else {
        return Ok(Vec::new());
    };
    if arr.is_empty() {
        return Ok(vec![Vec::new()]);
    }

    if matches!(arr.first(), Some(Value::Array(_))) {
        serde_json::from_value::<Vec<Vec<CourseInfo>>>(Value::Array(arr))
            .map_err(serde::de::Error::custom)
    } else {
        let inner: Vec<CourseInfo> =
            serde_json::from_value(Value::Array(arr)).map_err(serde::de::Error::custom)?;
        Ok(vec![inner])
    }
}

/// Field-level deserialization: converts `level_order` numbers or strings to strings,
/// uses `to_string()` for other types, and returns an empty array by default.
pub(crate) fn deserialize_level_order<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let values = Option::<Vec<Value>>::deserialize(deserializer)?.unwrap_or_default();
    Ok(values
        .into_iter()
        .map(|v| match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
            other => other.to_string(),
        })
        .collect())
}

/// Internal helper type: used to construct `CourseInfo` more simply and handle md5/sha256 lists.
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

        // Process charts and fill missing level with "0"
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
            extra: BTreeMap::new(),
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
            extra: BTreeMap::new(),
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

/// General helper to deserialize empty strings into `None`-like behavior.
pub(crate) fn de_numstring<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Value>::deserialize(deserializer)?;
    let Some(value) = opt else {
        return Err(serde::de::Error::custom(
            "expected string or number, found None",
        ));
    };
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        other => Err(serde::de::Error::custom(format!(
            "expected string or number, got {}",
            other
        ))),
    }
}
