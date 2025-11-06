//! BMS表格数据获取器示例程序
//!
//! 这个程序演示了如何使用 `bms_table` 库来获取和解析BMS表格数据。
//! 程序会从指定的网站获取BMS表格的HTML和JSON数据，并显示解析后的信息。
//!
//! # 功能
//!
//! - 从BMS表格网站获取数据
//! - 解析表格头信息和课程配置
//! - 显示谱面数据和歌曲信息
//! - 演示数据查找功能
//!
//! # 运行方式
//!
//! ```bash
//! # 使用默认URL
//! cargo run --example demo
//!
//! # 指定自定义URL
//! cargo run --example demo "https://stellabms.xyz/sl/table.html"
//! ```
//!
//! # 输出示例
//!
//! 程序运行后会显示类似以下的输出：
//!
//! ```
//! BMS表格数据获取器
//! ==================
//! 正在获取BMS表格数据...
//! URL: https://stellabms.xyz/sl/table.html
//!
//! 成功获取BMS表格数据
//!
//! 表格信息:
//!   名称: Satellite
//!   符号: sl
//!   数据URL: score.json
//!   课程数量: 1
//!   谱面数据数量: 4
//!
//! 课程信息:
//!   - Satellite Skill Analyzer 2nd sl0
//!     约束: ["grade_mirror", "gauge_lr2", "ln"]
//!     奖杯: [Trophy { name: "silvermedal", missrate: 5.0, scorerate: 70.0 }]
//!     MD5数量: 4
//!
//! 谱面数据 (前5个):
//!   1. "Fresco" [ANOTHER] - Lemi. obj:69 de 74
//!      MD5: 176c2b2db4efd66cf186caae7923d477
//!      URL: https://venue.bmssearch.net/bmsshuin3/75
//! ```
#![allow(unused)]

use anyhow::Result;
#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::fetch_bms_table;
use std::env;

/// 主函数
///
/// 演示BMS表格数据获取器的完整功能。
/// 程序会从指定的网站获取BMS表格数据，并显示解析后的信息。
///
/// # 参数
///
/// 可选的第一个命令行参数应该是BMS表格的URL。如果未提供，将使用默认URL。
///
/// # 返回值
///
/// 返回 `Result<()>`，如果成功则返回 `Ok(())`，否则返回错误。
///
/// # 错误处理
///
/// 如果获取数据失败，程序会显示错误信息并正常退出。
#[tokio::main]
#[cfg(feature = "reqwest")]
async fn main() -> Result<()> {
    // 显示程序标题
    println!("BMS表格数据获取器");
    println!("==================");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 确定要使用的URL
    let base_url = if args.len() > 1 {
        &args[1]
    } else {
        "https://stellabms.xyz/sl/table.html"
    };

    // 显示正在获取数据的信息
    println!("正在获取BMS表格数据...");
    println!("URL: {base_url}");

    // 获取完整的BMS表格数据
    let bms_table = fetch_bms_table(base_url).await.unwrap_or_else(|e| {
        println!("获取BMS表格数据失败: {e}");
        std::process::exit(1);
    });

    // 显示成功信息
    println!("\n成功获取BMS表格数据");

    // 显示表格基本信息
    println!("\n表格信息:");
    println!("  名称: {}", bms_table.header.name);
    println!("  符号: {}", bms_table.header.symbol);
    println!("  数据URL: {}", bms_table.header.data_url);
    println!("  课程数量: {}", bms_table.header.course.len());
    println!("  谱面数据数量: {}", bms_table.data.charts.len());

    // 显示课程信息
    println!("\n课程信息:");
    for course in bms_table.header.course.iter().flatten() {
        println!("  - {}", course.name);
        println!("    约束: {:?}", course.constraint);
        println!("    奖杯: {:?}", course.trophy);
        println!("    谱面数量: {}", course.charts.len());
    }

    // 显示前几个谱面数据
    println!("\n谱面数据 (前5个):");
    for (i, score) in bms_table.data.charts.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {}",
            i + 1,
            score.title.as_deref().unwrap_or(""),
            score.artist.as_deref().unwrap_or("")
        );
        println!("     MD5: {}", score.md5.as_deref().unwrap_or(""));
        println!("     URL: {}", score.url.as_deref().unwrap_or(""));
    }

    // 演示查找功能
    if let Some(first_score) = bms_table.data.charts.first() {
        println!("\n演示查找功能:");
        if let Some(found) = bms_table
            .data
            .charts
            .iter()
            .find(|score| score.md5 == first_score.md5)
        {
            println!(
                "  通过MD5找到: {} - {}",
                found.title.as_deref().unwrap_or(""),
                found.artist.as_deref().unwrap_or("")
            );
        }

        if let Some(found) = bms_table
            .data
            .charts
            .iter()
            .find(|score| score.sha256 == first_score.sha256)
        {
            println!(
                "  通过SHA256找到: {} - {}",
                found.title.as_deref().unwrap_or(""),
                found.artist.as_deref().unwrap_or("")
            );
        }
    }

    Ok(())
}

#[cfg(not(feature = "reqwest"))]
fn main() {}
