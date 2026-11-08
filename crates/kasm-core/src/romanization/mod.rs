//! 中古音罗马字方案共用的音节表与切分（通字 / unt / tupa）。
//!
//! 三套方案都是「字 → 一串音节」的罗马字，音节表由各自的 `data/*-syllables.txt`
//! 编译进二进制；这里只放与方案无关的查表、前缀判断与 DP 切分。

use std::cmp::Reverse;

use crate::parser::{ParseError, Segmentation, Syllable};

/// 切分上限，与粤拼一致。
const MAX_SEGMENTATIONS: usize = 8;

/// 每个位置最多留几种切法。
const MAX_PER_POSITION: usize = 8;

/// 一套方案的音节表（排好序、去重）。
pub struct Table {
    syllables: Vec<&'static str>,
}

impl Table {
    pub fn from_source(source: &'static str) -> Self {
        let mut syllables: Vec<&'static str> = source
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        syllables.sort_unstable();
        syllables.dedup();
        Self { syllables }
    }

    pub fn is_syllable(&self, text: &str) -> bool {
        self.syllables.binary_search(&text).is_ok()
    }

    /// `text` 是不是某个合法音节的**真**前缀。
    pub fn is_syllable_prefix(&self, text: &str) -> bool {
        self.syllables
            .iter()
            .any(|s| s.len() > text.len() && s.starts_with(text))
    }

    /// 把一段无分隔的罗马字切成音节。允许 `'` 强制分隔，末尾允许一个没打完的音节。
    /// 返回按「音节少、不完整音节少、前面的音节长」排序的切法，最多 [`MAX_SEGMENTATIONS`] 种。
    pub fn segment(&self, input: &str) -> Result<Vec<Segmentation>, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError::Empty);
        }
        // 各方案的音节可以是任意符号（unt 的 IPA、通用羅馬字的 ꝩ/ç/声调符），
        // 只拒绝空白与分隔符；大小写也按原样比较
        for (position, ch) in input.chars().enumerate() {
            if ch.is_whitespace() || ch == '/' {
                return Err(ParseError::InvalidChar {
                    position: position + 1,
                    ch,
                });
            }
        }
        let chunks: Vec<&str> = input
            .split('\'')
            .filter(|chunk| !chunk.is_empty())
            .collect();
        if chunks.is_empty() {
            return Err(ParseError::NoSegmentation);
        }
        let mut results = vec![Segmentation {
            syllables: Vec::new(),
        }];
        for (index, chunk) in chunks.iter().enumerate() {
            let is_last = index + 1 == chunks.len();
            let options = self.segment_chunk(chunk, is_last);
            if options.is_empty() {
                return Err(ParseError::NoSegmentation);
            }
            let mut next = Vec::with_capacity(results.len() * options.len());
            for base in &results {
                for option in &options {
                    let mut syllables = base.syllables.clone();
                    syllables.extend(option.syllables.iter().cloned());
                    next.push(Segmentation { syllables });
                }
            }
            results = next;
        }
        prune(&mut results);
        Ok(results)
    }

    fn segment_chunk(&self, chunk: &str, allow_partial: bool) -> Vec<Segmentation> {
        let n = chunk.len();
        let mut best: Vec<Vec<Vec<Syllable>>> = vec![Vec::new(); n + 1];
        best[0].push(Vec::new());
        for start in 0..=n {
            if !chunk.is_char_boundary(start) || best[start].is_empty() {
                continue;
            }
            let rest = &chunk[start..];
            for syllable in self
                .syllables
                .iter()
                .copied()
                .filter(|s| rest.starts_with(s))
            {
                let end = start + syllable.len();
                let mut candidates = best[start].clone();
                for candidate in &mut candidates {
                    candidate.push(Syllable::complete(syllable));
                }
                offer(&mut best[end], candidates);
            }
            if allow_partial && self.is_syllable_prefix(rest) {
                let mut candidates = best[start].clone();
                for candidate in &mut candidates {
                    candidate.push(Syllable::partial(rest));
                }
                offer(&mut best[n], candidates);
            }
        }
        std::mem::take(&mut best[n])
            .into_iter()
            .map(|syllables| Segmentation { syllables })
            .collect()
    }
}

/// 把一组切法并进目标位置：同样的音节序列只留一份，最多留 [`MAX_PER_POSITION`] 种。
fn offer(slot: &mut Vec<Vec<Syllable>>, candidates: Vec<Vec<Syllable>>) {
    for candidate in candidates {
        if slot.iter().any(|existing| *existing == candidate) {
            continue;
        }
        slot.push(candidate);
    }
    slot.sort_by_cached_key(|syllables| {
        let incomplete = syllables.iter().filter(|s| !s.complete).count();
        let first = syllables.first().map_or(0, |s| s.text.len());
        (syllables.len(), incomplete, Reverse(first))
    });
    slot.truncate(MAX_PER_POSITION);
}

fn prune(results: &mut Vec<Segmentation>) {
    results.sort_by_cached_key(|segmentation| {
        let incomplete = segmentation.incomplete_count();
        let first = segmentation
            .syllables
            .first()
            .map_or(0, |syllable| syllable.text.len());
        (segmentation.syllables.len(), incomplete, Reverse(first))
    });
    results.dedup();
    results.truncate(MAX_SEGMENTATIONS);
}
