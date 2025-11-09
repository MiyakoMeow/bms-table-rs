//! 序列化模块。
#![cfg(feature = "serde")]

use crate::{BmsTableHeader, ChartItem};
use serde::ser::SerializeMap;

impl serde::Serialize for BmsTableHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("symbol", &self.symbol)?;
        map.serialize_entry("data_url", &self.data_url)?;
        map.serialize_entry("course", &self.course)?;
        map.serialize_entry("level_order", &self.level_order)?;

        for (k, v) in &self.extra {
            map.serialize_entry(k, v)?;
        }

        map.end()
    }
}

impl serde::Serialize for ChartItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        // 必填字段
        map.serialize_entry("level", &self.level)?;

        // 可选字段：仅在存在时输出
        if let Some(ref md5) = self.md5 {
            map.serialize_entry("md5", md5)?;
        }
        if let Some(ref sha256) = self.sha256 {
            map.serialize_entry("sha256", sha256)?;
        }
        if let Some(ref title) = self.title {
            map.serialize_entry("title", title)?;
        }
        if let Some(ref subtitle) = self.subtitle {
            map.serialize_entry("subtitle", subtitle)?;
        }
        if let Some(ref artist) = self.artist {
            map.serialize_entry("artist", artist)?;
        }
        if let Some(ref subartist) = self.subartist {
            map.serialize_entry("subartist", subartist)?;
        }
        if let Some(ref url) = self.url {
            map.serialize_entry("url", url)?;
        }
        if let Some(ref url_diff) = self.url_diff {
            map.serialize_entry("url_diff", url_diff)?;
        }

        // 展平额外字段到顶层
        for (k, v) in &self.extra {
            map.serialize_entry(k, v)?;
        }

        map.end()
    }
}
