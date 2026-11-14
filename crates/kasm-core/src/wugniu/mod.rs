//! 吳語（混合腔 / rime-wugniu-suwu）：苏州吴语罗马字，输入时去掉声调数字。
//!
//! 音节表由 `data/wugniu-syllables.txt` 编译进二进制，与通字 / tupa 共用罗马字查询管线。

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::from_source(include_str!("../../data/wugniu-syllables.txt")));

pub fn table() -> &'static Table {
    &TABLE
}

pub fn is_syllable(text: &str) -> bool {
    TABLE.is_syllable(text)
}

pub fn is_syllable_prefix(text: &str) -> bool {
    TABLE.is_syllable_prefix(text)
}

pub fn segment(input: &str) -> Result<Vec<Segmentation>, ParseError> {
    TABLE.segment(input)
}
