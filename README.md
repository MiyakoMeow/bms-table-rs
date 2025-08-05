# BMS表格数据获取器

这是一个用Rust编写的BMS表格数据获取和解析工具，能够从指定的网站获取BMS表格的HTML和JSON数据，并解析其中的结构信息。

## 功能特性

- 🔍 从HTML页面中提取bmstable字段
- 📊 解析BMS表格头信息JSON
- 🎵 获取和解析谱面数据
- 🔎 支持通过MD5和SHA256查找谱面数据
- 📋 课程信息管理和查询
- 🏆 奖杯信息解析
- 🚀 提供多种异步API接口

## 项目结构

```
bms-table/
├── Cargo.toml          # 项目配置和依赖
├── src/
│   ├── lib.rs          # 核心库代码和API接口
│   ├── fetch.rs        # 数据获取和解析模块
│   └── main.rs         # 示例程序
├── examples/
│   └── demo.rs         # 函数演示示例
└── README.md           # 项目说明
```

## 数据结构

### BmsTable
完整的BMS表格数据，包含：
- `name`: 表格名称
- `symbol`: 表格符号
- `header_url`: 表格头文件URL
- `data_url`: 谱面数据文件URL
- `course`: 课程信息数组
- `charts`: 谱面数据数组

### BmsTableHeader
BMS表格的头信息，包含：
- `name`: 表格名称
- `symbol`: 表格符号
- `data_url`: 谱面数据文件URL
- `course`: 课程信息数组

### CourseInfo
课程信息，包含：
- `name`: 课程名称
- `constraint`: 约束条件
- `trophy`: 奖杯信息
- `charts`: 谱面数据列表（包含该课程的所有谱面信息）

### ChartItem
谱面数据项，包含：
- `level`: 难度等级
- `md5`: MD5哈希（可选）
- `sha256`: SHA256哈希（可选）
- `title`: 歌曲标题（可选）
- `subtitle`: 歌曲副标题（可选）
- `artist`: 艺术家（可选）
- `subartist`: 副艺术家（可选）
- `url`: 下载链接（可选）
- `url_diff`: 差分文件链接（可选）
- `extra`: 额外数据

## 使用方法

### 作为库使用

#### 方法1: 使用fetch_bms_table（推荐方式）

```rust
use bms_table::fetch_bms_table;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let base_url = "https://stellabms.xyz/sl/table.html";
    
    // 获取完整的BMS表格数据
    let bms_table = fetch_bms_table(base_url).await?;
    
    println!("表格名称: {}", bms_table.name);
    println!("谱面数据数量: {}", bms_table.charts.len());
    
    Ok(())
}
```

#### 方法2: 分步获取JSON数据（高级用法）

```rust
use bms_table::{fetch_table_json_data, create_bms_table_from_json};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 分步获取JSON数据
    let (header_url, header_json, data_json) = fetch_table_json_data("https://stellabms.xyz/sl/table.html").await?;
    let bms_table = create_bms_table_from_json(&header_url, header_json, data_json).await?;
    
    println!("表格名称: {}", bms_table.name);
    
    Ok(())
}
```

### 运行示例程序

```bash
# 运行主程序
cargo run

# 运行函数演示示例
cargo run --example demo
```

## API参考

### 主要异步函数

#### `fetch_bms_table(url: &str) -> Result<BmsTable>`
从URL直接获取完整的BmsTable对象，这是最简单的方式。

#### `fetch_table_json_data(url: &str) -> Result<(String, Value, Value)>`
从URL获取header的绝对URL地址、header和data的JSON解析树。

#### `create_bms_table_from_json(header_url: &str, header_json: Value, data_json: Value) -> Result<BmsTable>`
从header的绝对URL地址、header和data的JSON解析树创建BmsTable对象。

### 内部实现

BmsTableParser类现在是内部实现，不再对外公开。用户应该使用上面提到的公共API函数。

## 依赖项

- `reqwest` - HTTP客户端
- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
- `scraper` - HTML解析
- `anyhow` - 错误处理
- `url` - URL处理

## 构建和测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例程序
cargo run

# 运行函数演示
cargo run --example demo
```

## 示例输出

运行示例程序会显示类似以下的输出：

```
BMS表格数据获取器
==================
正在获取BMS表格数据...
URL: https://stellabms.xyz/sl/table.html

✅ 成功获取BMS表格数据!

📋 表格信息:
  名称: Satellite
  符号: sl
  数据URL: score.json
  课程数量: 13
  谱面数据数量: 1986

🎵 课程信息:
  - Satellite Skill Analyzer 2nd sl0
    约束: ["grade_mirror", "gauge_lr2", "ln"]
    奖杯: [Trophy { name: "silvermedal", missrate: 5.0, scorerate: 70.0 }, Trophy { name: "goldmedal", missrate: 2.5, scorerate: 85.0 }]
    谱面数量: 4
  - Satellite Skill Analyzer 2nd sl1
    约束: ["grade_mirror", "gauge_lr2", "ln"]
    奖杯: [Trophy { name: "silvermedal", missrate: 5.0, scorerate: 70.0 }, Trophy { name: "goldmedal", missrate: 2.5, scorerate: 85.0 }]
    谱面数量: 4
  ... (更多课程)

📊 谱面数据 (前5个):
  1. "Fresco" [ANOTHER] - Lemi. obj:69 de 74
     MD5: 176c2b2db4efd66cf186caae7923d477
     URL: https://venue.bmssearch.net/bmsshuin3/75
  2. -Never ending journey- [BLACKANOTHER] - SOMON
  3. -終天- [BLACK ANOTHER] - SOMON
  4. 2anyFirst [7-A] - Sobrem
     MD5: f5456ea7a63431ce7575d2583fcf9c68
     URL: http://manbow.nothing.sh/event/event.cgi?action=More_def&num=209&event=127

🔍 演示查找功能:
  通过MD5找到: "Fresco" [ANOTHER] - Lemi. obj:69 de 74
  通过SHA256找到: "Fresco" [ANOTHER] - Lemi. obj:69 de 74
```

## 特性说明

### 空字符串处理
ChartItem中的可选字段在解析时会自动将空字符串转换为None，确保数据的准确性。

### 异步支持
所有API都是异步的，支持高效的并发操作。

### 数据转换
CourseInfo结构体支持多种数据格式的自动转换：
- 如果JSON中包含 `md5` 字段（MD5哈希列表），会自动转换为 `charts` 中的 `ChartItem`
- 如果JSON中包含 `sha256` 字段（SHA256哈希列表），会自动转换为 `charts` 中的 `ChartItem`
- 转换后的 `ChartItem` 使用默认的 `level: "0"`，其他字段为 `None`

### 错误处理
使用anyhow进行统一的错误处理，提供清晰的错误信息。

## 许可证

MIT License 