# BMS表格数据获取器

这是一个用Rust编写的BMS表格数据获取和解析工具，能够从指定的网站获取BMS表格的HTML和JSON数据，并解析其中的结构信息。

## 功能特性

- 🔍 从HTML页面中提取bmstable字段
- 📊 解析BMS表格头信息JSON
- 🎵 获取和解析分数数据
- 🔎 支持通过MD5和SHA256查找分数数据
- 📋 课程信息管理和查询
- 🏆 奖杯信息解析

## 项目结构

```
bms-table/
├── Cargo.toml          # 项目配置和依赖
├── src/
│   ├── lib.rs          # 核心库代码
│   └── main.rs         # 示例程序
└── README.md           # 项目说明
```

## 数据结构

### BmsTableHeader
BMS表格的头信息，包含：
- `name`: 表格名称
- `symbol`: 表格符号
- `data_url`: 分数数据文件URL
- `course`: 课程信息数组

### CourseInfo
课程信息，包含：
- `name`: 课程名称
- `constraint`: 约束条件
- `trophy`: 奖杯信息
- `md5`: MD5哈希列表

### ScoreItem
分数数据项，包含：
- `id`: 唯一标识符
- `md5`: MD5哈希
- `sha256`: SHA256哈希
- `title`: 歌曲标题
- `artist`: 艺术家
- `url`: 下载链接
- `url_diff`: 差分文件链接
- `level`: 难度等级

## 使用方法

### 作为库使用

```rust
use bms_table::BmsTableParser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let parser = BmsTableParser::new();
    let base_url = "https://stellabms.xyz/sl/table.html";
    
    // 获取完整的BMS表格数据
    let (header, scores) = parser.fetch_complete_table(base_url).await?;
    
    // 查找特定MD5的分数数据
    if let Some(score) = parser.find_score_by_md5(&scores, "your_md5_here") {
        println!("找到歌曲: {} - {}", score.title, score.artist);
    }
    
    Ok(())
}
```

### 运行示例程序

```bash
cargo run
```

## API参考

### BmsTableParser

#### 构造函数
- `new()` - 创建新的解析器实例

#### 主要方法
- `extract_bmstable_url(html_url)` - 从HTML页面提取bmstable URL
- `get_table_header(header_url)` - 获取并解析表格头信息
- `get_score_data(score_url)` - 获取并解析分数数据
- `fetch_complete_table(base_url)` - 完整的获取流程

#### 查找方法
- `find_score_by_md5(scores, md5)` - 通过MD5查找分数数据
- `find_score_by_sha256(scores, sha256)` - 通过SHA256查找分数数据
- `find_course_by_name(header, name)` - 通过名称查找课程

#### 辅助方法
- `get_all_courses(header)` - 获取所有课程信息

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
  课程数量: 1
  分数数据数量: 4

🎵 课程信息:
  - Satellite Skill Analyzer 2nd sl0
    约束: ["grade_mirror", "gauge_lr2", "ln"]
    奖杯: [Trophy { name: "silvermedal", missrate: 5.0, scorerate: 70.0 }, Trophy { name: "goldmedal", missrate: 2.5, scorerate: 85.0 }]
    MD5数量: 4

📊 分数数据 (前5个):
  1. "Fresco" [ANOTHER] - Lemi. obj:69 de 74
     MD5: 176c2b2db4efd66cf186caae7923d477
     URL: https://venue.bmssearch.net/bmsshuin3/75
```

## 许可证

MIT License 