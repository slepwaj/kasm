//! Xina 拼音（传统派普通话方案）：无调拉丁拼式，由普通话词库转换而来。
//!
//! 音节表由 `data/xina-syllables.txt` 编译进二进制；与通字 / tupa / 上古音共用一条罗马字查询管线。
//!

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::from_source(include_str!("../../data/xina-syllables.txt")));

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

/// 候选行显示用；新方案无调，原样返回（保留旧声调符号的兼容处理）。
pub fn display_reading(reading: &str) -> String {
    reading
        .split_whitespace()
        .map(display_syllable)
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_syllable(syllable: &str) -> String {
    let Some(last) = syllable.chars().last() else {
        return String::new();
    };
    let tone = match last {
        '!' => 0,
        '@' => 1,
        '#' => 2,
        '$' => 3,
        '%' => 4,
        _ => return syllable.to_owned(),
    };
    let base = &syllable[..syllable.len() - last.len_utf8()];
    let Some((index, vowel)) = base
        .char_indices()
        .find(|(_, character)| matches!(character, 'a' | 'e' | 'i' | 'o' | 'u'))
    else {
        return syllable.to_owned();
    };
    let marked = match (vowel, tone) {
        ('a', 1) => 'á',
        ('a', 2) => 'ǎ',
        ('a', 3) => 'à',
        ('a', 4) => 'ạ',
        ('e', 1) => 'é',
        ('e', 2) => 'ě',
        ('e', 3) => 'è',
        ('e', 4) => 'ẹ',
        ('i', 1) => 'í',
        ('i', 2) => 'ǐ',
        ('i', 3) => 'ì',
        ('i', 4) => 'ị',
        ('o', 1) => 'ó',
        ('o', 2) => 'ǒ',
        ('o', 3) => 'ò',
        ('o', 4) => 'ọ',
        ('u', 1) => 'ú',
        ('u', 2) => 'ǔ',
        ('u', 3) => 'ù',
        ('u', 4) => 'ụ',
        _ => vowel,
    };
    let mut display = String::with_capacity(base.len() + 1);
    display.push_str(&base[..index]);
    display.push(marked);
    display.push_str(&base[index + vowel.len_utf8()..]);
    display
}
