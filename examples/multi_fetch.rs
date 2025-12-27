//! Concurrent multi-table fetching example
//!
//! This example demonstrates fetching multiple BMS difficulty tables concurrently and emitting events when each table finishes.
//! It uses asynchronous concurrency to process multiple tables in parallel for better efficiency.
//!
//! # Features
//!
//! - Concurrently fetch multiple BMS difficulty tables
//! - Emit an event when each table finishes
//! - Display fetch progress and results
//! - Error handling and retry mechanics
#![cfg_attr(not(feature = "reqwest"), allow(unused_imports))]

use anyhow::Result;
use bms_table::BmsTable;
#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::Fetcher;
use std::env;
#[cfg(feature = "reqwest")]
use tokio::sync::mpsc;
#[cfg(feature = "reqwest")]
use url::Url;

/// Main function
///
/// Demonstrates the full functionality of concurrent fetching across multiple difficulty tables.
/// Fetches multiple BMS difficulty tables simultaneously and emits an event when each one finishes.
///
/// # Parameters
///
/// Optional command-line arguments should be a list of BMS difficulty table URLs. If not provided, a default list is used.
///
/// # Return value
///
/// Returns `Result<()>`; `Ok(())` on success, otherwise an error.
///
/// # Error handling
///
/// If fetching fails for one table, the program prints the error and continues with others.
#[tokio::main]
#[cfg(feature = "reqwest")]
async fn main() -> Result<()> {
    // Display program title
    println!("Concurrent difficulty-table fetcher");
    println!("===================");

    // Create a reusable fetcher
    let fetcher = Fetcher::lenient()?;

    // Display fetching information
    let urls = table_urls();
    let url_count = urls.len();
    println!("Fetching {url_count} difficulty tables...");
    println!();

    // Create a channel for event handling
    let (tx, mut rx) = mpsc::channel::<FetchResult>(100);

    // Start the event handler task
    let event_handler = tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result.table {
                Ok(table) => {
                    println!(
                        "{} fetched successfully ({} charts, {} course groups, {} courses)",
                        result.name,
                        table.data.charts.len(),
                        table.header.course.len(),
                        table.header.course.iter().flatten().count()
                    );
                }
                Err(e) => {
                    println!("{} fetch failed: {}", result.name, e);
                }
            }
        }
    });

    // Fetch all tables concurrently
    let fetch_tasks: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let tx = tx.clone();
            let fetcher_cloned = fetcher.clone();
            tokio::spawn(async move {
                let result = fetch_single_table(&fetcher_cloned, &url).await;
                let _ = tx.send(result).await;
            })
        })
        .collect();

    // Wait for all fetch tasks to finish
    for task in fetch_tasks {
        let _ = task.await;
    }

    // Close the sender and wait for the event handler to finish
    drop(tx);
    let _ = event_handler.await;

    // Display summary
    println!();
    println!("Fetch summary:");
    println!("  Concurrency: {url_count} tables");
    println!("  Processing: async concurrency");
    println!("  Event handling: real-time dispatch");

    Ok(())
}

#[cfg(feature = "reqwest")]
/// Get the list of URLs to use
fn table_urls() -> Vec<Url> {
    // Read command-line arguments
    let args: Vec<String> = env::args().collect();

    // Determine the URL list to use

    if args.len() > 1 {
        args.iter()
            .skip(1)
            .filter_map(|s| Url::parse(s).ok())
            .collect()
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
            "http://onzonium.at-ninja.jp/dp/",
            "https://lr2.sakura.ne.jp/overjoy.php",
        ]
        .into_iter()
        .filter(|url| !url.is_empty())
        .filter_map(|s| Url::parse(s).ok())
        .collect()
    }
}

/// Fetch result for a difficulty table
#[derive(Debug)]
#[cfg(feature = "reqwest")]
struct FetchResult {
    /// Table name
    name: String,
    /// Result of fetching the table
    table: anyhow::Result<BmsTable>,
}

/// Fetch a single difficulty table
#[cfg(feature = "reqwest")]
async fn fetch_single_table(fetcher: &Fetcher, url: &Url) -> FetchResult {
    match fetcher.fetch_table(url.clone()).await {
        Ok(fetched) => {
            let bms_table = fetched.table;
            FetchResult {
                name: bms_table.header.name.clone(),
                table: Ok(bms_table),
            }
        }
        Err(e) => FetchResult {
            name: url.to_string(),
            table: Err(e),
        },
    }
}

#[cfg(not(feature = "reqwest"))]
fn main() {}
