//! BMS表格数据获取器
//!
//! 这个程序接受一个URL参数，获取BMS表格数据并打印BmsTable对象。
//!
//! # 使用方法
//!
//! ```bash
//! cargo run "https://stellabms.xyz/sl/table.html"
//! ```
//!
//! # 功能
//!
//! - 从命令行参数获取URL
//! - 获取并解析BMS表格数据
//! - 打印完整的BmsTable对象

#[cfg(feature = "reqwest")]
use anyhow::{Context, Result};
#[cfg(feature = "reqwest")]
use std::env;

/// 主函数
///
/// 从命令行参数获取URL，获取BMS表格数据并打印BmsTable对象。
///
/// # 参数
///
/// 第一个命令行参数应该是BMS表格的URL。
///
/// # 返回值
///
/// 返回 `Result<()>`，如果成功则返回 `Ok(())`，否则返回错误。
///
/// # 错误处理
///
/// 如果没有提供URL参数或获取数据失败，程序会显示错误信息并退出。
#[tokio::main]
#[cfg(feature = "reqwest")]
async fn main() -> Result<()> {
    use bms_table::fetch_bms_table;
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    let url = if args.len() > 1 {
        &args[1]
    } else {
        "http://zris.work/bmstable/pms_upper/header.json"
    };

    println!("正在获取BMS表格数据...");
    println!("URL: {url}");

    // 获取BMS表格数据
    let bms_table = fetch_bms_table(url).await.context("获取BMS表格数据失败")?;

    println!("✅ 成功获取BMS表格数据!");
    println!();

    // 打印完整的BmsTable对象
    println!("📋 BmsTable对象:");
    println!("{bms_table:#?}");

    Ok(())
}
