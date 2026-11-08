//! tupa（中古音罗马字）：`古籍字頻統計_注音.xlsx` 的 tupa 列。
//!
//! 纯 ASCII 拼式（`tjypu` 之类），音节表由 `data/tupa-syllables.txt` 编译进二进制。

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::from_source(include_str!("../../data/tupa-syllables.txt")));

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
