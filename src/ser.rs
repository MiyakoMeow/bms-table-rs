//! 序列化模块。
#![cfg(feature = "serde")]

use crate::BmsTableHeader;
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

// ChartItem now derives `Serialize` in `lib.rs`, with `serde(flatten)` for `extra`.
