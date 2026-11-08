//! 上古音（Shanggu）：`上古汉语.xlsx` 的字表，Baxter–Sagart 式拼写。
//!
//! 输入侧做了归一：去掉 `C.` 前缀与 `'`，只留 `a`–`z`（`C.ra` → `ra`、`l'ik` → `lik`）。
//! 音节表由 `data/shanggu-syllables.txt` 编译进二进制。

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::from_source(include_str!("../../data/shanggu-syllables.txt")));

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
