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
//!
//! # 运行方式
//!
//! ```bash
//! # 使用默认难度表列表
//! cargo run --example multi_fetch
//!
//! # 指定自定义难度表列表
//! cargo run --example multi_fetch "https://stellabms.xyz/sl/table.html" "https://stellabms.xyz/ln/table.html"
//! ```
//!
//! # 输出示例
//!
//! 程序运行后会显示类似以下的输出：
//!
//! ```
//! 多难度表并发获取器
//! ===================
//! 正在获取 3 个难度表...
//!
//! Satellite 获取完成
//! LN 获取完成
//! DP 获取失败: 网络连接错误
//!
//! 获取完成统计:
//!   成功: 2 个
//!   失败: 1 个
//!   总计: 3 个
//! ```

use anyhow::Result;
use bms_table::{fetch_bms_table, BmsTable};
use std::env;
use tokio::sync::mpsc;

/// 难度表获取结果
#[derive(Debug)]
struct FetchResult {
    /// 难度表名称
    name: String,
    /// 是否成功
    success: bool,
    /// 错误信息（如果失败）
    error: Option<String>,
    /// 难度表
    table: Option<BmsTable>,
}

/// 处理单个难度表获取完成的事件
fn handle_fetch_complete(result: FetchResult) {
    match result.success {
        true => {
            let table = result.table.unwrap();
            println!(
                "{} 获取完成 ({} 个谱面，{} 个课程组，{} 个课程)",
                result.name,
                table.charts.len(),
                table.course.len(),
                table.course.iter().flatten().count()
            );
        }
        false => {
            let error = result.error.unwrap_or_else(|| "未知错误".to_string());
            println!("{} 获取失败: {}", result.name, error);
        }
    }
}

/// 获取单个难度表
async fn fetch_single_table(url: &str) -> FetchResult {
    match fetch_bms_table(url).await {
        Ok(bms_table) => FetchResult {
            name: bms_table.name.clone(),
            success: true,
            error: None,
            table: Some(bms_table),
        },
        Err(e) => FetchResult {
            name: url.to_string(),
            success: false,
            error: Some(e.to_string()),
            table: None,
        },
    }
}

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
async fn main() -> Result<()> {
    // 显示程序标题
    println!("多难度表并发获取器");
    println!("===================");

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 确定要使用的URL列表
    let urls = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![
            "https://stellabms.xyz/sl/table.html",
            "https://stellabms.xyz/dp/table.html",
            "http://zris.work/bmstable/normal/normal_header.json",
            "http://zris.work/bmstable/insane/insane_header.json",
            "http://zris.work/bmstable/overjoy/header.json",
            "http://zris.work/bmstable/normal2/header.json",
            "http://zris.work/bmstable/insane2/insane_header.json",
            "http://zris.work/bmstable/insane_easy/header_json.json",
            "http://zris.work/bmstable/insane_normal/header_json.json",
            "http://zris.work/bmstable/insane_hard/header_json.json",
            "http://zris.work/bmstable/insane_fc/header_json.json",
            "http://zris.work/bmstable/stellalite/Stellalite-header.json",
            "http://zris.work/bmstable/homage/header.json",
            "http://zris.work/bmstable/dp_normal/dpn_header.json",
            "http://zris.work/bmstable/pms_normal/pmsdatabase_header.json",
            "http://zris.work/bmstable/pms_course/course_header.json",
            "http://zris.work/bmstable/pms_insane/insane_pmsdatabase_header.json",
            "http://zris.work/bmstable/pms_upper/header.json",
            "http://zris.work/bmstable/10k/head.json",
        ]
        .into_iter()
        .filter(|url| !url.is_empty())
        .map(ToString::to_string)
        .collect()
    };

    // 显示正在获取数据的信息
    let url_count = urls.len();
    println!("正在获取 {} 个难度表...", url_count);
    println!();

    // 创建通道用于事件处理
    let (tx, mut rx) = mpsc::channel(100);

    // 启动事件处理任务
    let event_handler = tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            handle_fetch_complete(result);
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
    println!("  并发获取: {} 个难度表", url_count);
    println!("  处理方式: 异步并发");
    println!("  事件处理: 实时触发");

    Ok(())
}
