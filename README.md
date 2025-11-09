# BMS 难度表数据获取与解析库

使用 Rust 实现的 BMS 难度表数据获取与解析库。支持从网页或头部 JSON 构建完整数据结构，覆盖表头、课程、奖杯与谱面条目等。

## 功能特性

- 从 HTML `<meta name="bmstable">` 提取头部 JSON 地址（启用 `scraper` 特性）
- 解析表头 JSON 为 `BmsTableHeader`，未识别字段保留在 `extra`
- 解析谱面数据为 `BmsTableData`，兼容数组与 `{ charts: [...] }` 两种格式
- 将课程中的 `md5`/`sha256` 列表自动转换为 `ChartItem`
- 一站式网络获取 API（启用 `reqwest` 特性）

默认启用 `reqwest` 特性；如需 HTML 解析，请启用 `scraper` 特性（`reqwest` 特性已包含 `scraper`）。

## 快速开始

```rust
use anyhow::Result;
use bms_table::fetch::reqwest::fetch_table;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://stellabms.xyz/sl/table.html";
    let table = fetch_table(url).await?;
    println!("{}: {} charts", table.header.name, table.data.charts.len());
    Ok(())
}
```

## 项目结构

```text
bms-table-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 核心数据结构与crate级文档
│   ├── fetch.rs            # HTML解析与响应解析（需scraper特性）
│   └── fetch/reqwest.rs    # 网络获取接口（默认启用reqwest特性）
├── examples/
│   ├── demo.rs             # 单表获取与展示
│   └── multi_fetch.rs      # 并发获取多个难度表
└── tests/                  # 单元测试
```

## API 概览

- `BmsTable`：顶层数据结构，包含 `header` 与 `data`
- `BmsTableHeader`：表头元数据；未识别字段保留在 `extra`
- `BmsTableData`：谱面数据数组
- `CourseInfo`：课程信息，支持 `md5`/`sha256` 列表自动转换为谱面
- `ChartItem`：谱面条目，空字符串在反序列化时自动转换为 `None`
- `Trophy`：奖杯要求（最大 miss 率、最低得分率）
- `fetch::reqwest::fetch_table(url)`：从网页或头部 JSON 源拉取并解析完整表
- `fetch::get_web_header_json_value(s)`：将响应字符串解析为头部 JSON 或其 URL
- `fetch::extract_bmstable_url(html)`：从 HTML 中提取 bmstable 头部地址

## 特性说明

- `reqwest`：网络获取接口（默认启用）
- `scraper`：HTML 解析与头部地址提取

## 最低支持的 Rust 版本

- `rustc 1.78.0`
