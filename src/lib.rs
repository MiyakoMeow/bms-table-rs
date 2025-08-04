//! 示例程序
//! 

#![warn(missing_docs)]

use url::Url;

use crate::fetch::{CourseInfo, ScoreItem};

pub mod fetch;

/// BMS难度表数据，看这一个就够了
#[derive(Debug, Clone, PartialEq)]
pub struct BmsTable {
    /// 表格名称，如 "Satellite"
    pub name: String,
    /// 表格符号，如 "sl"
    pub symbol: String,
    /// 表格头文件的相对URL，如 "header.json"
    pub header_url: Url,
    /// 分数数据文件的相对URL，如 "score.json"
    pub data_url: Url,
    /// 课程信息数组，每个元素是一个课程组的数组
    pub course: Vec<Vec<CourseInfo>>,
    /// 分数数据
    pub scores: Vec<ScoreItem>,
}