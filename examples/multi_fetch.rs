//! 多难度表并发获取示例程序
//!
//! 这个程序演示了如何同时获取多个BMS难度表，并在每个难度表获取完成后触发事件。
//! 程序使用异步并发来同时处理多个难度表的获取，提高效率。
//!
//! # 功能
//!
//! - 并发获取多个BMS难度表
//! - 每个难度表获取完成后触发事件
//! - 显示获取进度和结果
//! - 错误处理和重试机制
#![cfg_attr(not(feature = "reqwest"), allow(unused_imports))]

use anyhow::Result;
use bms_table::BmsTable;
#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::fetch_table;
use std::env;
#[cfg(feature = "reqwest")]
use tokio::sync::mpsc;

/// 主函数
///
/// 演示多难度表并发获取的完整功能。
/// 程序会同时获取多个BMS难度表，并在每个难度表获取完成后触发事件。
///
/// # 参数
///
/// 可选的命令行参数应该是BMS难度表的URL列表。如果未提供，将使用默认URL列表。
///
/// # 返回值
///
/// 返回 `Result<()>`，如果成功则返回 `Ok(())`，否则返回错误。
///
/// # 错误处理
///
/// 如果获取数据失败，程序会显示错误信息但继续处理其他难度表。
#[tokio::main]
#[cfg(feature = "reqwest")]
async fn main() -> Result<()> {
    // 显示程序标题
    println!("多难度表并发获取器");
    println!("===================");

    // 显示正在获取数据的信息
    let urls = table_urls();
    let url_count = urls.len();
    println!("正在获取 {url_count} 个难度表...");
    println!();

    // 创建通道用于事件处理
    let (tx, mut rx) = mpsc::channel::<FetchResult>(100);

    // 启动事件处理任务
    let event_handler = tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result.table {
                Ok(table) => {
                    println!(
                        "{} 获取完成 ({} 个谱面，{} 个课程组，{} 个课程)",
                        result.name,
                        table.data.charts.len(),
                        table.header.course.len(),
                        table.header.course.iter().flatten().count()
                    );
                }
                Err(e) => {
                    println!("{} 获取失败: {}", result.name, e);
                }
            }
        }
    });

    // 并发获取所有难度表
    let fetch_tasks: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = fetch_single_table(&url).await;
                let _ = tx.send(result).await;
            })
        })
        .collect();

    // 等待所有获取任务完成
    for task in fetch_tasks {
        let _ = task.await;
    }

    // 关闭发送端，等待事件处理完成
    drop(tx);
    let _ = event_handler.await;

    // 显示统计信息
    println!();
    println!("获取完成统计:");
    println!("  并发获取: {url_count} 个难度表");
    println!("  处理方式: 异步并发");
    println!("  事件处理: 实时触发");

    Ok(())
}

#[cfg(feature = "reqwest")]
/// 获取要使用的URL列表
fn table_urls() -> Vec<String> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 确定要使用的URL列表

    if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![
            "https://stellabms.xyz/sl/table.html",
            "https://stellabms.xyz/dp/table.html",
            "https://zris.work/bmstable/normal/normal_header.json",
            "https://zris.work/bmstable/insane/insane_header.json",
            "http://rattoto10.jounin.jp/table_overjoy.html",
            "http://rattoto10.jounin.jp/table.html",
            "http://rattoto10.jounin.jp/table_insane.html",
            "http://walkure.net/hakkyou/for_glassist/bms/?lamp=easy",
            "http://walkure.net/hakkyou/for_glassist/bms/?lamp=normal",
            "http://walkure.net/hakkyou/for_glassist/bms/?lamp=hard",
            "http://walkure.net/hakkyou/for_glassist/bms/?lamp=fc",
            "https://notmichaelchen.github.io/stella-table-extensions/stellalite.html",
            "https://iidxtool.kasacontent.com/homage/table.php",
            "https://zris.work/bmstable/dp_normal/dpn_header.json", // TODO
            "https://pmsdifficulty.xxxxxxxx.jp/PMSdifficulty.html",
            "https://pmsdifficulty.xxxxxxxx.jp/insane_PMSdifficulty.html",
            "http://zris.work/bmstable/pms_insane/insane_pmsdatabase_header.json",
            "https://pmsdifficulty.xxxxxxxx.jp/_pastoral_insane_table.html",
            "https://www.notepara.com/glassist/10k",
        ]
        .into_iter()
        .filter(|url| !url.is_empty())
        .map(ToString::to_string)
        .collect()
    }
}

/// 难度表获取结果
#[derive(Debug)]
#[cfg(feature = "reqwest")]
struct FetchResult {
    /// 难度表名称
    name: String,
    /// 难度表获取结果
    table: anyhow::Result<BmsTable>,
}

/// 获取单个难度表
#[cfg(feature = "reqwest")]
async fn fetch_single_table(url: &str) -> FetchResult {
    match fetch_table(url).await {
        Ok(bms_table) => FetchResult {
            name: bms_table.header.name.clone(),
            table: Ok(bms_table),
        },
        Err(e) => FetchResult {
            name: url.to_string(),
            table: Err(e),
        },
    }
}

#[cfg(not(feature = "reqwest"))]
fn main() {}
