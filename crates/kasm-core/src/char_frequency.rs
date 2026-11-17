//! 汉字通用频率：给罗马字长句的字形消歧用。
//!
//! 通字 / unt / GCR 的读音表只给同音字，古音词频又常把「友」排在「有」前面；
//! 整句 Viterbi 里加一点现代汉字频率的先验，长句才不会全按生僻同音字拼。

use std::collections::HashMap;
use std::sync::LazyLock;

/// 汉字 → 词频（从 `assets/lexicon/dict.tsv` 的通用词表汇总，只收单字）。
static FREQUENCY: LazyLock<HashMap<char, f64>> = LazyLock::new(|| {
    include_str!("../data/char-frequency.tsv")
        .lines()
        .filter_map(|line| {
            let (character, frequency) = line.split_once('\t')?;
            let character = character.chars().next()?;
            let frequency = frequency.trim().parse::<f64>().ok()?;
            Some((character, frequency))
        })
        .collect()
});

/// 多字词 → 通字词库词频；古音长句优先认成词，而不是拆成同音字。
static WORDS: LazyLock<HashMap<&'static str, f64>> = LazyLock::new(|| {
    include_str!("../data/tungdzih-word-frequency.tsv")
        .lines()
        .filter_map(|line| {
            let (word, frequency) = line.split_once('\t')?;
            Some((word, frequency.trim().parse::<f64>().ok()?))
        })
        .collect()
});

/// 一个词的平均 log 频率；表里没有的字按最低频算。
pub fn word_prior(word: &str) -> f64 {
    let mut count = 0.0;
    let mut total = 0.0;
    for character in word.chars() {
        // 标点、空格、emoji 不参与。
        if !character.is_alphanumeric() && !('\u{3400}'..='\u{9fff}').contains(&character) {
            continue;
        }
        let frequency = FREQUENCY.get(&character).copied().unwrap_or(1.0);
        total += (frequency + 1.0).ln();
        count += 1.0;
    }
    if count == 0.0 { 0.0 } else { total / count }
}

/// 多字词的通用词频对数；单字为 0。
pub fn word_probability(word: &str) -> f64 {
    if word.chars().count() < 2 {
        return 0.0;
    }
    WORDS
        .get(word)
        .copied()
        .map_or(0.0, |frequency| (frequency + 1.0).ln())
}
