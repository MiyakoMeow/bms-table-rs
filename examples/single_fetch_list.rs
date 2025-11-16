//! Single-run example for fetching the table list: uses fetch_table_list_full
//!
//! How to run:
//! - With the default `reqwest` feature enabled, run:
//!   `cargo run --example single_fetch_list`
//! - If default features are disabled, explicitly enable:
//!   `cargo run --example single_fetch_list --features reqwest`

#![cfg_attr(not(feature = "reqwest"), allow(unused_imports))]

#[cfg(feature = "reqwest")]
use bms_table::fetch::reqwest::{fetch_table_list_full, make_lenient_client};

#[cfg(feature = "reqwest")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "https://script.google.com/macros/s/AKfycbzaQbcI9UZDcDlSHHl2NHilhmePrNrwxRdOFkmIXsfnbfksKKmAB3V65WZ8jPWU-7E/exec?table=tablelist";

    let client = make_lenient_client()?;
    let (listes, raw) = fetch_table_list_full(&client, url).await?;
    println!("Fetched {} table list entries.", listes.len());

    for (i, item) in listes.iter().take(10).enumerate() {
        println!("#{i}: {} [{}] -> {}", item.name, item.symbol, item.url);
    }

    println!("Raw JSON length: {}", raw.len());
    Ok(())
}

#[cfg(not(feature = "reqwest"))]
fn main() {
    eprintln!("This example requires the `reqwest` feature to be enabled.");
}
