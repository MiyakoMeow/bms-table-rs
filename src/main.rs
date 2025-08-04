//! BMS表格数据获取器示例程序
//!
//! 这个程序演示了如何使用 `bms_table` 库来获取和解析BMS表格数据。
//! 程序会从指定的网站获取BMS表格的HTML和JSON数据，并显示解析后的信息。
//!
//! # 功能
//!
//! - 从BMS表格网站获取数据
//! - 解析表格头信息和课程配置
//! - 显示分数数据和歌曲信息
//! - 演示数据查找功能
//!
//! # 运行方式
//!
//! ```bash
//! cargo run
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
//! ✅ 成功获取BMS表格数据!
//!
//! 📋 表格信息:
//!   名称: Satellite
//!   符号: sl
//!   数据URL: score.json
//!   课程数量: 1
//!   分数数据数量: 4
//!
//! 🎵 课程信息:
//!   - Satellite Skill Analyzer 2nd sl0
//!     约束: ["grade_mirror", "gauge_lr2", "ln"]
//!     奖杯: [Trophy { name: "silvermedal", missrate: 5.0, scorerate: 70.0 }]
//!     MD5数量: 4
//!
//! 📊 分数数据 (前5个):
//!   1. "Fresco" [ANOTHER] - Lemi. obj:69 de 74
//!      MD5: 176c2b2db4efd66cf186caae7923d477
//!      URL: https://venue.bmssearch.net/bmsshuin3/75
//! ```

#![warn(missing_docs)]

pub mod fetch;

use anyhow::Result;
use bms_table::fetch_bms_table;

/// 主函数
///
/// 演示BMS表格数据获取器的完整功能。
/// 程序会从指定的网站获取BMS表格数据，并显示解析后的信息。
///
/// # 返回值
///
/// 返回 `Result<()>`，如果成功则返回 `Ok(())`，否则返回错误。
///
/// # 错误处理
///
/// 如果获取数据失败，程序会显示错误信息并正常退出。
#[tokio::main]
async fn main() -> Result<()> {
    // 显示程序标题
    println!("BMS表格数据获取器");
    println!("==================");

    let base_url = "https://stellabms.xyz/sl/table.html";

    // 显示正在获取数据的信息
    println!("正在获取BMS表格数据...");
    println!("URL: {base_url}");

    // 获取完整的BMS表格数据
    let bms_table = fetch_bms_table(base_url).await.unwrap_or_else(|e| {
        println!("❌ 获取BMS表格数据失败: {e}");
        std::process::exit(1);
    });

    // 显示成功信息
    println!("\n✅ 成功获取BMS表格数据!");

    // 显示表格基本信息
    println!("\n📋 表格信息:");
    println!("  名称: {}", bms_table.name);
    println!("  符号: {}", bms_table.symbol);
    println!("  数据URL: {}", bms_table.data_url);
    println!("  课程数量: {}", bms_table.course.len());
    println!("  分数数据数量: {}", bms_table.scores.len());

    // 显示课程信息
    println!("\n🎵 课程信息:");
    for course in bms_table.course.iter().flatten() {
        println!("  - {}", course.name);
        println!("    约束: {:?}", course.constraint);
        println!("    奖杯: {:?}", course.trophy);
        println!("    MD5数量: {}", course.md5.len());
    }

    // 显示前几个分数数据
    println!("\n📊 分数数据 (前5个):");
    for (i, score) in bms_table.scores.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {}",
            i + 1,
            score.title.as_ref().unwrap_or(&"".to_string()),
            score.artist.as_ref().unwrap_or(&"".to_string())
        );
        println!(
            "     MD5: {}",
            score.md5.as_ref().unwrap_or(&"".to_string())
        );
        println!(
            "     URL: {}",
            score.url.as_ref().unwrap_or(&"".to_string())
        );
    }

    // 演示查找功能
    if let Some(first_score) = bms_table.scores.first() {
        println!("\n🔍 演示查找功能:");
        if let Some(found) = bms_table
            .scores
            .iter()
            .find(|score| score.md5 == first_score.md5)
        {
            println!(
                "  通过MD5找到: {} - {}",
                found.title.as_ref().unwrap_or(&"".to_string()),
                found.artist.as_ref().unwrap_or(&"".to_string())
            );
        }

        if let Some(found) = bms_table
            .scores
            .iter()
            .find(|score| score.sha256 == first_score.sha256)
        {
            println!(
                "  通过SHA256找到: {} - {}",
                found.title.as_ref().unwrap_or(&"".to_string()),
                found.artist.as_ref().unwrap_or(&"".to_string())
            );
        }
    }

    Ok(())
}
