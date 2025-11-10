//! 单次抓取索引示例：使用 fetch_table_list_full
//!
//! 运行方式：
//! - 默认启用 `reqwest` 特性，可直接运行：
//!   `cargo run --example single_fetch_index`
//! - 若禁用了默认特性，请显式启用：
//!   `cargo run --example single_fetch_index --features reqwest`

#![cfg_attr(not(feature = "reqwest"), allow(unused_imports))]

#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::{fetch_table_list_full, make_lenient_client};

#[cfg(feature = "reqwest")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "https://script.google.com/macros/s/AKfycbzaQbcI9UZDcDlSHHl2NHilhmePrNrwxRdOFkmIXsfnbfksKKmAB3V65WZ8jPWU-7E/exec?table=tablelist";

    let client = make_lenient_client()?;
    let (indexes, raw) = fetch_table_list_full(&client, url).await?;
    println!("Fetched {} index entries.", indexes.len());

    for (i, item) in indexes.iter().take(10).enumerate() {
        println!("#{i}: {} [{}] -> {}", item.name, item.symbol, item.url);
    }

    println!("Raw JSON length: {}", raw.len());
    Ok(())
}

#[cfg(not(feature = "reqwest"))]
fn main() {
    eprintln!("This example requires the `reqwest` feature to be enabled.");
}
