//! Xina 切拼（中古音）：来自《常用字表_Xina切拼.xlsx》的单字读音。
//!
//! 纯 ASCII 拼式，音节表由 `data/xina-qiepin-syllables.txt` 编译进二进制；
//! 与通字 / tupa / 上古音共用一条罗马字查询管线。

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> = LazyLock::new(|| {
    Table::from_source(include_str!("../../data/xina-qiepin-syllables.txt"))
});

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
