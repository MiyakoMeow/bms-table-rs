//! BMS表格函数演示示例
//!
//! 这个示例演示了如何使用新添加的三个异步函数：
//! 1. create_bms_table_from_json - 从JSON数据创建BmsTable对象
//! 2. fetch_table_json_data - 从URL获取JSON数据
//! 3. fetch_bms_table - 从URL直接获取BmsTable对象

use anyhow::Result;
use bms_table::{create_bms_table_from_json, fetch_bms_table, fetch_table_json_data};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("BMS表格函数演示");
    println!("================");

    // 示例1: 从JSON数据创建BmsTable对象
    println!("\n1. 从JSON数据创建BmsTable对象:");
    let header_url = "https://example.com/header.json";
    let header_json = json!({
        "name": "Test Table",
        "symbol": "test",
        "data_url": "score.json",
        "course": []
    });
    let data_json = json!([
        {
            "level": "1",
            "title": "Test Song",
            "artist": "Test Artist",
            "md5": "test123456789"
        }
    ]);

    match create_bms_table_from_json(header_url, header_json, data_json).await {
        Ok(bms_table) => {
            println!("✅ 成功创建BmsTable对象");
            println!("   表格名称: {}", bms_table.name);
            println!("   表格符号: {}", bms_table.symbol);
            println!("   分数数据数量: {}", bms_table.scores.len());
        }
        Err(e) => {
            println!("❌ 创建BmsTable对象失败: {}", e);
        }
    }

    // 示例2: 从URL获取JSON数据
    println!("\n2. 从URL获取JSON数据:");
    let test_url = "https://stellabms.xyz/sl/table.html";

    match fetch_table_json_data(test_url).await {
        Ok((header_url, header_json, data_json)) => {
            println!("✅ 成功获取JSON数据");
            println!("   Header URL: {}", header_url);
            println!(
                "   Header JSON 键: {:?}",
                header_json
                    .as_object()
                    .map(|obj| obj.keys().collect::<Vec<_>>())
            );
            println!(
                "   Data JSON 数组长度: {}",
                data_json.as_array().map(|arr| arr.len()).unwrap_or(0)
            );
        }
        Err(e) => {
            println!("❌ 获取JSON数据失败: {}", e);
        }
    }

    // 示例3: 从URL直接获取BmsTable对象
    println!("\n3. 从URL直接获取BmsTable对象:");

    match fetch_bms_table(test_url).await {
        Ok(bms_table) => {
            println!("✅ 成功获取BmsTable对象");
            println!("   表格名称: {}", bms_table.name);
            println!("   表格符号: {}", bms_table.symbol);
            println!("   课程数量: {}", bms_table.course.len());
            println!("   分数数据数量: {}", bms_table.scores.len());

            // 显示前几个分数数据
            println!("   前3个分数数据:");
            for (i, score) in bms_table.scores.iter().take(3).enumerate() {
                println!(
                    "     {}. {} - {} [{}]",
                    i + 1,
                    score.title.as_ref().unwrap_or(&"未知标题".to_string()),
                    score.artist.as_ref().unwrap_or(&"未知艺术家".to_string()),
                    score.level
                );
            }
        }
        Err(e) => {
            println!("❌ 获取BmsTable对象失败: {}", e);
        }
    }

    println!("\n演示完成!");
    Ok(())
}
