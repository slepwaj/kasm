//! 单字声调表 `data/tones.tsv`（`kasm-dict-convert tones` 从通用词表生成）。
//!
//! 词库不带声调，查词仍按无调拼音；`[general] show_tones` 给拼音行补声调时，词库没标调的字查这张表。
//! 表里一个字的读音按通用词表排序号首次出现的顺序排，第一个就是最常见的。

use std::collections::HashMap;
use std::sync::LazyLock;

/// 字 → [(无调音节, 声调位集：第 n 位表示 n 声，轻声是 5)]，外加最常见的读音（无调音节, 声调）。
type Table = HashMap<char, (Vec<(&'static str, u8)>, (&'static str, u8))>;

static TABLE: LazyLock<Table> = LazyLock::new(|| parse(include_str!("../data/tones.tsv")));

fn parse(source: &'static str) -> Table {
    let mut table = Table::new();
    for line in source.lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((word, readings)) = line.split_once('\t') else {
            continue;
        };
        let Some(ch) = word.chars().next() else {
            continue;
        };
        let mut syllables: Vec<(&'static str, u8)> = Vec::new();
        let mut first = None;
        for reading in readings.split(' ') {
            let Some(tone) = reading
                .chars()
                .last()
                .and_then(|c| c.to_digit(10))
                .and_then(|d| u8::try_from(d).ok())
            else {
                continue;
            };
            let syllable = &reading[..reading.len() - 1];
            first.get_or_insert((syllable, tone));
            match syllables.iter_mut().find(|(s, _)| *s == syllable) {
                Some((_, mask)) => *mask |= 1 << tone,
                None => syllables.push((syllable, 1 << tone)),
            }
        }
        if let Some(first) = first {
            table.insert(ch, (syllables, first));
        }
    }
    table
}

/// `ch` 读 `syllable` 时的声调：是它最常见的读音就取那个声调，否则这个音只有一种声调才取；拿不准是 0。
pub fn usual(ch: char, syllable: &str) -> u8 {
    let Some((readings, first)) = TABLE.get(&ch) else {
        return 0;
    };
    if first.0 == syllable {
        return first.1;
    }
    match readings.iter().find(|(s, _)| *s == syllable) {
        Some((_, mask)) if mask.count_ones() == 1 => mask.trailing_zeros() as u8,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usual_tone_of_a_character() {
        assert_eq!(usual('的', "de"), 5);
        assert_eq!(usual('的', "di"), 0);
        assert_eq!(usual('好', "hao"), 3);
        assert_eq!(usual('好', "xyz"), 0);
    }

    #[test]
    fn parses_first_reading_and_masks() {
        let table = parse("# 注释\n的\tde5 di4 di2\n");
        let (readings, first) = &table[&'的'];
        assert_eq!(*first, ("de", 5));
        assert_eq!(*readings, [("de", 1 << 5), ("di", (1 << 4) | (1 << 2))]);
    }
}
