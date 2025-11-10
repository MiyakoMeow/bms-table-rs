# BMS 难度表数据获取与解析库

使用 Rust 实现的 BMS 难度表获取与解析库。支持从网页或头部 JSON 构建完整数据结构，覆盖表头、课程、奖杯与谱面条目，并提供难度表列表获取能力。

## 安装

在 `Cargo.toml` 中添加依赖（默认启用 `serde`、`scraper`、`reqwest`）：

```toml
[dependencies]
bms-table = "0.6"
```

禁用默认特性并按需启用：

```toml
[dependencies]
bms-table = { version = "0.6", default-features = false, features = ["serde", "scraper"] }
```

## 功能特性

- 从 HTML `<meta name="bmstable">` 提取头部 JSON 地址（启用 `scraper`）。
- 解析表头 JSON 为 `BmsTableHeader`，未识别字段保留到 `extra`。
- 解析谱面数据为 `BmsTableData`，兼容纯数组与 `{ charts: [...] }` 两种格式。
- 将课程中的 `md5`/`sha256` 列表自动转换为 `ChartItem`，缺失 `level` 时补为 "0"。
- 一站式网络获取 API（启用 `reqwest`，隐式启用 `scraper`）。
- 获取难度表列表。

## 特性开关

- `serde`：类型的序列化/反序列化支持（默认启用）。
- `scraper`：HTML 解析与 bmstable 头部地址提取（默认启用；`reqwest` 隐式启用）。
- `reqwest`：网络获取实现（默认启用；需要 `tokio` 运行时）。

## API 概览

- `BmsTable`：顶层数据结构，包含 `header` 与 `data`。
- `BmsTableHeader`：表头元数据；未识别字段保留到 `extra`。
- `BmsTableData`：谱面数据数组或 `{ charts }` 透明包装。
- `CourseInfo`：课程信息，支持 `md5`/`sha256` 列表自动转换为谱面。
- `ChartItem`：谱面条目；空字符串在反序列化时自动转换为 `None`。
- `Trophy`：奖杯要求（最大 miss 率、最低得分率）。
- `fetch::reqwest::fetch_table(url)`：从网页或头部 JSON 源拉取并解析完整表。
- `fetch::reqwest::fetch_table_full(url)`：同时返回原始头部与数据 JSON 文本。
- `fetch::reqwest::fetch_table_list(url)`：获取难度表列表。
- `fetch::reqwest::fetch_table_list_full(url)`：返回列表项与原始 JSON 文本。
- `fetch::get_web_header_json_value(str)`：将响应字符串解析为头部 JSON 或其 URL（`HeaderQueryContent`）。
- `fetch::extract_bmstable_url(html)`：从 HTML 中提取 bmstable 头部地址。

## 示例程序

- `examples/single_fetch_index.rs`：单次抓取难度表列表并打印前若干条目。
- `examples/multi_fetch.rs`：并发抓取多个难度表并输出进度与结果。

## 文档与链接

- `docs.rs`：https://docs.rs/bms-table
- 仓库：https://github.com/MiyakoMeow/bms-table-rs

## 许可

本项目基于 Apache-2.0 许可证开源。
