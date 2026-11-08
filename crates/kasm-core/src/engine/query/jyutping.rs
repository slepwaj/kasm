//! 粤拼查询：无调 Jyutping 切分 + 词库前缀查询。
//!
//! 与形码一样，这是一条不经过普通话音节表的旁路：没有简拼、模糊音、拼写纠错、整句与中英混输，
//! 候选按词频、同一输入串下的选择次数与上屏上下文排序；译词、学习、日志与统计照常共用。

use std::time::Instant;

use crate::engine::{Learner, MAX_CANDIDATES, choice_key};
use crate::ranking::Scored;
use crate::{
    Candidate, CandidateKind, CandidateList, Engine, ParseError, Query, Timings, jyutping, ranking,
    sentence,
};

impl Engine {
    /// 粤拼候选。切分出来的第一个读法（音节最少、前面的音节最长）决定查词模式：
    /// 末尾音节没打完按前缀查，打完按整词查；更长的词由前缀匹配带出来。
    pub(super) fn query_jyutping(
        &self,
        keys: &str,
        rest: String,
        start: Instant,
    ) -> Result<Query, ParseError> {
        let segmentations = jyutping::segment(keys)?;
        let parse = start.elapsed();

        let start = Instant::now();
        let best = &segmentations[0];
        let patterns = best.patterns();
        let coverage = best.letters();
        let full_last = best
            .syllables
            .last()
            .is_some_and(|syllable| syllable.complete);
        let log_total = (self.total_frequency() as f64).max(1.0).ln();
        let hits = self.dictionary.lookup_pattern(&patterns);
        let mut scored: Vec<Scored<'_>> = hits
            .into_iter()
            .map(|hit| Scored {
                hit,
                full_last,
                coverage,
                abbreviated: 0,
                weight: self.learner.weight(hit.text),
                penalty: 0.0,
                split: false,
            })
            .collect();
        let lookup = start.elapsed();

        let start = Instant::now();
        let letters = choice_key(keys, keys.len());
        ranking::rank(&mut scored, MAX_CANDIDATES, |item| {
            let choice = letters
                .get(..item.coverage)
                .map_or(0, |input| self.learner.choice_weight(input, item.hit.text));
            let log_prob = sentence::transition_log_prob(
                &*self.jyutping_language_model,
                self.personal(),
                self.chain.context(),
                item.hit.text,
                sentence::fallback_log_prob(item.hit.frequency, item.hit.secondary, log_total),
            );
            (choice, log_prob)
        });
        // 每个候选都附上它的粤拼（候选窗右侧的 annotation）：词级取命中的键，整句取路径音节。
        // 词库不带声调，显示无调粤拼（`nei hou`），与敲法一致。
        let mut items: Vec<Candidate> = scored
            .iter()
            .map(|item| {
                let mut candidate = super::chinese_candidate(item, None);
                candidate.reading = Some(item.hit.syllables().collect::<Vec<_>>().join(" "));
                candidate
            })
            .collect();
        // 整句：至少两个词、没有占位音节，且与词级候选不重复时插在最前（空格上屏的就是它）。
        // 一条路径只解释了一个词时不出——那与词级候选是同一件事。
        if best.syllables.len() >= 2 {
            let positions: Vec<Vec<kasm_dictionary::SyllablePattern<'_>>> =
                patterns.iter().map(|pattern| vec![*pattern]).collect();
            let dictionaries = self.all_dictionaries();
            let input = sentence::SearchInput::new(
                &dictionaries,
                &*self.jyutping_language_model,
                self.personal(),
                |text| self.learner.weight(text),
                |_, _| 0.0,
            )
            .with_limits(self.search_limits);
            let conversion =
                sentence::convert(&input, &positions, &mut self.span_cache.borrow_mut());
            if let Some(conversion) = conversion
                && conversion.word_count() >= 2
                && !conversion.has_placeholder()
                && !items.iter().any(|item| item.text == conversion.text)
            {
                items.insert(
                    0,
                    Candidate {
                        text: conversion.text,
                        kind: CandidateKind::Sentence,
                        reading: Some(conversion.syllables.join(" ")),
                        syllables: conversion.syllables,
                        translation: None,
                        aux_code: None,
                    },
                );
            }
        }
        let rank = start.elapsed();

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
            typed_display: None,
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
