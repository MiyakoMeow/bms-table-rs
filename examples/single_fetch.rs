//! Example: fetch a single BMS difficulty table and print summary
#![cfg_attr(not(feature = "reqwest"), allow(unused_imports))]

use anyhow::Result;
use std::env;

#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::Fetcher;
#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
#[tokio::main]
async fn main() -> Result<()> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "https://stellabms.xyz/sl/table.html".to_string());
    let url = Url::parse(&url)?;

    let fetcher = Fetcher::lenient()?;

    match fetcher.fetch_table(url.clone()).await {
        Ok(table) => {
            println!(
                "{} fetched successfully ({} charts, {} course groups, {} courses)",
                table.header.name,
                table.data.charts.len(),
                table.header.course.len(),
                table.header.course.iter().flatten().count()
            );
        }
        Err(e) => {
            eprintln!("Fetch failed for: {}", url);
            eprintln!("Message: {}", e);
            eprintln!("Causes:");
            for (i, cause) in e.chain().enumerate() {
                eprintln!("  [{}] {}", i, cause);
            }
            match std::env::var("RUST_BACKTRACE").as_deref() {
                Ok("1") => {
                    eprintln!("Backtrace:");
                    eprintln!("{:?}", e.backtrace());
                }
                _ => {
                    eprintln!("Hint: set RUST_BACKTRACE=1 to print backtrace.");
                }
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "reqwest"))]
fn main() {
    eprintln!("This example requires the `reqwest` feature to be enabled.");
}
