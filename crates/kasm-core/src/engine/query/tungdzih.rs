//! 通字查询：Tungdzih 罗马字切分 + 词库前缀查询。
//!
//! 与粤拼一样，这是一条不经过普通话音节表的旁路：没有简拼、拼写纠错与中英混输，
//! 但有各方案自己的自定义模糊规则；整句用普通话音模型。通字单字母音节多（`u` 烏、`a` 啊），同一串输入常有多种切法
//! （`uea` = `u ea` 烏鴉 或 `ue a` 委啊），所以词级候选与整句都遍历全部切法再排序，
//! 不能只看第一种。

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::engine::{Learner, MAX_CANDIDATES, choice_key};
use crate::ranking::{self, Scored, FUZZY_PENALTY};
use crate::{
    Candidate, CandidateKind, CandidateList, Engine, ParseError, Query, Timings, romanization,
    sentence,
};

/// 通字 / unt / GCR 这类罗马字方案的自定义模糊规则：两边可以是声母、韵母或整音节。
/// 与拼音的 `CustomRule` 分开，因为 `dj`、`ꝩ` 这些不是拼音声母，不能用拼音表验证。
struct RomanRule {
    from: String,
    to: String,
}

impl RomanRule {
    fn parse(text: &str) -> Option<Self> {
        let (from, to) = text.split_once('=')?;
        let from = from.trim();
        let to = to.trim();
        if from.is_empty() || to.is_empty() || from == to {
            return None;
        }
        Some(Self {
            from: from.to_owned(),
            to: to.to_owned(),
        })
    }

    fn alternatives(&self, table: &romanization::Table, syllable: &str) -> Vec<String> {
        let mut out = Vec::new();
        for (from, to) in [(&self.from, &self.to), (&self.to, &self.from)] {
            if syllable == from {
                out.push(to.clone());
            }
            if let Some(rest) = syllable.strip_prefix(from.as_str()) {
                out.push(format!("{to}{rest}"));
            }
            if let Some(head) = syllable.strip_suffix(from.as_str()) {
                out.push(format!("{head}{to}"));
            }
        }
        out.retain(|candidate| candidate != syllable && table.is_syllable(candidate));
        out.sort();
        out.dedup();
        out
    }
}

impl Engine {
    /// 中古音罗马字（通字 / unt / tupa）候选。每种切法各自查词，合起来按词频、选择习惯与上下文排序；
    /// 整句在全部切法里取得分最高的一条。
    pub(super) fn query_romanized(
        &self,
        table: &'static romanization::Table,
        keys: &str,
        rest: String,
        start: Instant,
    ) -> Result<Query, ParseError> {
        let segmentations = table.segment(keys)?;
        let parse = start.elapsed();

        let start = Instant::now();
        let log_total = (self.total_frequency() as f64).max(1.0).ln();
        let rules: Vec<RomanRule> = self
            .fuzzy
            .custom
            .iter()
            .filter_map(|text| RomanRule::parse(text))
            .collect();
        let mut scored: Vec<Scored<'_>> = Vec::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let mut typed_for_hit: HashMap<(String, String), Vec<String>> = HashMap::new();
        for segmentation in &segmentations {
            let typed: Vec<String> = segmentation
                .syllables
                .iter()
                .map(|syllable| syllable.text.clone())
                .collect();
            let mut forms: Vec<Vec<String>> = segmentation
                .syllables
                .iter()
                .map(|syllable| vec![syllable.text.clone()])
                .collect();
            let mut fuzzy = false;
            if !rules.is_empty() {
                for (index, syllable) in segmentation.syllables.iter().enumerate() {
                    if !syllable.complete {
                        continue;
                    }
                    for rule in &rules {
                        for alternative in
                            rule.alternatives(table, &syllable.text)
                        {
                            if !forms[index].contains(&alternative) {
                                forms[index].push(alternative);
                                fuzzy = true;
                            }
                        }
                    }
                }
            }
            let patterns: Vec<Vec<kasm_dictionary::SyllablePattern<'_>>> = forms
                .iter()
                .zip(&segmentation.syllables)
                .map(|(forms, syllable)| {
                    forms
                        .iter()
                        .map(|text| kasm_dictionary::SyllablePattern {
                            text,
                            complete: syllable.complete,
                        })
                        .collect()
                })
                .collect();
            let coverage = segmentation.letters();
            let full_last = segmentation
                .syllables
                .last()
                .is_some_and(|syllable| syllable.complete);
            let hits = if fuzzy && full_last {
                self.dictionary.lookup_exact_alt(&patterns)
            } else {
                self.dictionary.lookup_pattern(&segmentation.patterns())
            };
            for hit in hits {
                let reading = hit.syllables().collect::<Vec<_>>().join(" ");
                if !seen.insert((hit.text.to_owned(), reading.clone())) {
                    continue;
                }
                typed_for_hit
                    .entry((hit.text.to_owned(), reading))
                    .or_insert_with(|| typed.clone());
                scored.push(Scored {
                    hit,
                    full_last,
                    coverage,
                    abbreviated: 0,
                    weight: self.learner.weight(hit.text),
                    penalty: if fuzzy { FUZZY_PENALTY } else { 0.0 },
                    split: false,
                });
            }
        }
        let lookup = start.elapsed();

        let start = Instant::now();
        let letters = choice_key(keys, keys.len());
        ranking::rank(&mut scored, MAX_CANDIDATES, |item| {
            let choice = letters
                .get(..item.coverage)
                .map_or(0, |input| self.learner.choice_weight(input, item.hit.text));
            let log_prob = sentence::transition_log_prob(
                &*self.language_model,
                self.personal(),
                self.chain.context(),
                item.hit.text,
                sentence::fallback_log_prob(item.hit.frequency, item.hit.secondary, log_total),
            );
            (choice, log_prob)
        });
        // 每个候选都附上它的通字（候选窗右侧的 annotation）：词级取命中的键，整句取路径音节
        let mut items: Vec<Candidate> = scored
            .iter()
            .map(|item| {
                let mut candidate = super::chinese_candidate(item, None);
                let reading = item.hit.syllables().collect::<Vec<_>>().join(" ");
                let typed = typed_for_hit
                    .get(&(item.hit.text.to_owned(), reading.clone()))
                    .map(|typed| typed[..candidate.syllables.len().min(typed.len())].to_vec());
                let display = typed.clone().unwrap_or_else(|| {
                    item.hit.syllables().map(str::to_owned).collect::<Vec<_>>()
                });
                candidate.syllables = display
                    .iter()
                    .cloned()
                    .chain(candidate.syllables.iter().skip(display.len()).cloned())
                    .collect();
                let display = candidate.syllables.join(" ");
                candidate.reading = Some(if self.xina {
                    crate::xina::display_reading(&display)
                } else {
                    display
                });
                candidate
            })
            .collect();
        // 整句：至少两个词、没有占位音节，且与词级候选不重复时插在最前（空格上屏的就是它）。
        // 各切法都转一次，取得分最高的那条——`uea` 要能选出「烏鴉」而不是「委啊」。
        // 但词级里已经有词完整覆盖输入时不再插整句：那种情况整句只是把词拆开重拼，反而挡路。
        let has_full_word = items.iter().any(|item| {
            item.kind == CandidateKind::Chinese
                && segmentations.iter().any(|segmentation| {
                    segmentation.syllables.len() == item.syllables.len()
                        && segmentation
                            .syllables
                            .iter()
                            .zip(&item.syllables)
                            .all(|(syllable, text)| syllable.text == *text)
                })
        });
        let mut best_conversion: Option<sentence::Conversion> = None;
        for segmentation in &segmentations {
            if segmentation.syllables.len() < 2 {
                continue;
            }
            let positions: Vec<Vec<kasm_dictionary::SyllablePattern<'_>>> = segmentation
                .patterns()
                .iter()
                .map(|pattern| vec![*pattern])
                .collect();
            let dictionaries = self.all_dictionaries();
            let prior = |_: usize, word: &str, _: &[String]| {
                if keys.len() > 16 {
                    0.35 * crate::char_frequency::word_prior(word)
                        + 0.4 * crate::char_frequency::word_probability(word)
                } else {
                    0.0
                }
            };
            let input = sentence::SearchInput::new(
                &dictionaries,
                &*self.language_model,
                self.personal(),
                |text| self.learner.weight(text),
                |_, _| 0.0,
            )
            .with_emission(&prior)
            .with_limits(self.search_limits);
            let conversion =
                sentence::convert(&input, &positions, &mut self.span_cache.borrow_mut());
            if let Some(mut conversion) = conversion {
                // 古音/通字词库里单字频率常压过多字词，长句会退化成逐字拼。
                // 在整句同一条评分上给多字词一点“成词”加分，避免 `ta men` 被拆成
                // `tam en` 这类切法抢走；只加在罗马字整句，不改普通话管线。
                let multi_char = conversion
                    .words
                    .iter()
                    .map(|word| word.text.chars().count().saturating_sub(1))
                    .sum::<usize>();
                conversion.score += if keys.len() > 16 {
                    40.0 * multi_char as f64
                } else {
                    0.0
                };
                if conversion.word_count() >= 2
                    && !conversion.has_placeholder()
                    && best_conversion
                        .as_ref()
                        .is_none_or(|best| conversion.score > best.score)
                {
                    best_conversion = Some(conversion);
                }
            }
        }
        if let Some(conversion) = best_conversion
            && !has_full_word
            && !items.iter().any(|item| item.text == conversion.text)
        {
            items.insert(
                0,
                Candidate {
                    text: conversion.text,
                    kind: CandidateKind::Sentence,
                    reading: Some(if self.xina {
                        crate::xina::display_reading(&conversion.syllables.join(" "))
                    } else {
                        conversion.syllables.join(" ")
                    }),
                    syllables: conversion.syllables,
                    translation: None,
                    aux_code: None,
                },
            );
        }
        let rank = start.elapsed();

        let typed_display = if self.xina {
            Some(
                segmentations
                    .first()
                    .map(|segmentation| {
                        segmentation
                            .syllables
                            .iter()
                            .map(|syllable| crate::xina::display_reading(&syllable.text))
                            .collect::<Vec<_>>()
                            .join("'")
                    })
                    .unwrap_or_else(|| crate::xina::display_reading(keys)),
            )
        } else {
            None
        };

        Ok(Query {
            segmentations,
            spelled: 0,
            candidates: CandidateList { items },
            // 切分已经覆盖整段输入：没有普通话音节表那种「切不动的尾巴」
            tail: String::new(),
            text: self.composition.text().to_owned(),
            cursor: self.composition.cursor(),
            rest,
            decoded_keys: false,
            shuangpin_raw_preedit: false,
            shuangpin_raw_window: false,
            typed_display,
            typed_case: None,
            pinyin_choices: Vec::new(),
            correction: None,
            aux: None,
            held: String::new(),
            timings: Timings {
                parse,
                lookup,
                rank,
            },
        })
    }
}
