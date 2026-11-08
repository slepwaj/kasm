//! 粤拼（Jyutping）：无调音节表与切分。
//!
//! 粤拼不进普通话音节表那条路：音节表、切分、查询各走一条旁路（与形码类似），
//! 普通话音节表（[`crate::parser`]）保持原样。数据来源见 `assets/lexicon/jyutping.tsv`。
//!
//! 音节表是 729 个**无调**粤拼音节（不含声调数字），来自 CanCLID Rime Cantonese 方案。

use std::cmp::Reverse;

use crate::parser::{ParseError, Segmentation, Syllable};

/// 729 个无调粤拼音节，按字节序排好（二分查找用）。
pub const SYLLABLES: &[&str] = &[
    "a", "aa", "aai", "aak", "aam", "aan", "aang", "aap", "aat", "aau", "ai", "ak", "am", "an",
    "ang", "ap", "at", "au", "baa", "baai", "baak", "baan", "baang", "baat", "baau", "bai", "bak",
    "bam", "ban", "bang", "bat", "bau", "be", "bei", "bek", "beng", "bet", "bi", "bik", "bin",
    "bing", "bit", "biu", "bo", "bok", "bon", "bong", "bou", "bui", "buk", "bun", "bung", "but",
    "caa", "caai", "caak", "caam", "caan", "caang", "caap", "caat", "caau", "cai", "cak", "cam",
    "can", "cang", "cap", "cat", "cau", "ce", "cek", "ceng", "ceoi", "ceon", "ceot", "cet", "ci",
    "cik", "cim", "cin", "cing", "cip", "cit", "ciu", "co", "coek", "coeng", "coet", "coi", "cok",
    "cong", "cou", "cuk", "cung", "cyu", "cyun", "cyut", "daa", "daai", "daak", "daam", "daan",
    "daang", "daap", "daat", "dai", "dak", "dam", "dan", "dang", "dap", "dat", "dau", "de", "dei",
    "dek", "deng", "deoi", "deon", "deot", "dep", "det", "deu", "di", "dik", "dim", "din", "ding",
    "dip", "dit", "diu", "do", "doe", "doek", "doeng", "doi", "dok", "dong", "dou", "duk", "dung",
    "dyun", "dyut", "e", "ei", "et", "faa", "faai", "faak", "faan", "faat", "fai", "fan", "fang",
    "fat", "fau", "fe", "fei", "fi", "fik", "fing", "fit", "fiu", "fo", "fok", "fong", "fu", "fui",
    "fuk", "fun", "fung", "fut", "ga", "gaa", "gaai", "gaak", "gaam", "gaan", "gaang", "gaap",
    "gaat", "gaau", "gai", "gak", "gam", "gan", "gang", "gap", "gat", "gau", "ge", "gei", "geng",
    "geoi", "gep", "gi", "gik", "gim", "gin", "ging", "gip", "git", "giu", "go", "goe", "goek",
    "goeng", "goi", "gok", "gon", "gong", "got", "gou", "gu", "gui", "guk", "gun", "gung", "gut",
    "gwaa", "gwaai", "gwaak", "gwaan", "gwaang", "gwaat", "gwai", "gwak", "gwan", "gwang", "gwat",
    "gwe", "gwei", "gwek", "gwi", "gwik", "gwing", "gwit", "gwo", "gwok", "gwong", "gyun", "gyut",
    "haa", "haai", "haak", "haam", "haan", "haang", "haap", "haau", "hai", "hak", "ham", "han",
    "hang", "hap", "hat", "hau", "he", "hei", "hek", "heng", "heoi", "hi", "hik", "him", "hin",
    "hing", "hip", "hit", "hiu", "hm", "hng", "ho", "hoe", "hoeng", "hoi", "hok", "hon", "hong",
    "hot", "hou", "huk", "hung", "hyun", "hyut", "ik", "it", "jaa", "jaai", "jaak", "jaang",
    "jaap", "jaau", "jai", "jam", "jan", "jap", "jat", "jau", "je", "jen", "jeng", "jeoi", "jeon",
    "ji", "jik", "jim", "jin", "jing", "jip", "jit", "jiu", "jo", "joek", "joeng", "jou", "juk",
    "jung", "jyu", "jyun", "jyut", "kaa", "kaai", "kaak", "kaam", "kaan", "kaang", "kaat", "kaau",
    "kai", "kak", "kam", "kan", "kang", "kap", "kat", "kau", "ke", "kei", "kek", "kem", "keng",
    "keoi", "kep", "ket", "keu", "ki", "kik", "kim", "kin", "king", "kip", "kit", "kiu", "ko",
    "koe", "koek", "koeng", "koi", "kok", "kong", "ku", "kui", "kuk", "kung", "kut", "kwaa",
    "kwaai", "kwaak", "kwaang", "kwai", "kwan", "kwang", "kwik", "kwok", "kwong", "kyun", "kyut",
    "la", "laa", "laai", "laak", "laam", "laan", "laang", "laap", "laat", "laau", "lai", "lak",
    "lam", "lan", "lang", "lap", "lat", "lau", "le", "lei", "lek", "lem", "leng", "leoi", "leon",
    "leot", "leu", "li", "lik", "lim", "lin", "ling", "lip", "lit", "liu", "lo", "loe", "loek",
    "loeng", "loi", "lok", "long", "lou", "lu", "luk", "lung", "lyun", "lyut", "m", "ma", "maa",
    "maai", "maak", "maan", "maang", "maat", "maau", "mai", "mak", "mam", "man", "mang", "mat",
    "mau", "me", "mei", "meng", "met", "meu", "mi", "mik", "min", "ming", "mit", "miu", "mo",
    "mok", "mon", "mong", "mou", "mu", "mui", "muk", "mun", "mung", "mut", "naa", "naai", "naam",
    "naan", "naap", "naat", "naau", "nai", "nak", "nam", "nan", "nang", "nap", "nat", "nau", "ne",
    "nei", "neng", "neoi", "neot", "ng", "ngaa", "ngaai", "ngaak", "ngaam", "ngaan", "ngaang",
    "ngaap", "ngaat", "ngaau", "ngai", "ngak", "ngam", "ngan", "ngang", "ngap", "ngat", "ngau",
    "nge", "ngek", "nget", "ngit", "ngo", "ngoi", "ngok", "ngon", "ngong", "ngot", "ngou", "ni",
    "nik", "nim", "nin", "ning", "nip", "nit", "niu", "no", "noek", "noeng", "noi", "nok", "nong",
    "nou", "nuk", "nung", "nyun", "o", "oet", "oi", "ok", "on", "ong", "ou", "paa", "paai", "paak",
    "paan", "paang", "paat", "paau", "pai", "pan", "pang", "pat", "pau", "pe", "pei", "pek",
    "peng", "pet", "pik", "pin", "ping", "pit", "piu", "po", "poi", "pok", "pong", "pou", "pui",
    "puk", "pun", "pung", "put", "saa", "saai", "saak", "saam", "saan", "saang", "saap", "saat",
    "saau", "sai", "sak", "sam", "san", "sang", "sap", "sat", "sau", "se", "sei", "sek", "seng",
    "seoi", "seon", "seot", "si", "sik", "sim", "sin", "sing", "sip", "sit", "siu", "so", "soe",
    "soek", "soeng", "soi", "sok", "song", "sou", "suk", "sung", "syu", "syun", "syut", "taa",
    "taai", "taam", "taan", "taap", "taat", "tai", "tam", "tan", "tang", "tap", "tau", "tek",
    "teng", "teoi", "teon", "teot", "tet", "ti", "tik", "tim", "tin", "ting", "tip", "tit", "tiu",
    "to", "toe", "toi", "tok", "tong", "tou", "tuk", "tung", "tyun", "tyut", "uk", "ung", "waa",
    "waai", "waak", "waan", "waang", "waat", "wai", "wak", "wan", "wang", "wat", "wau", "we",
    "wet", "wi", "wik", "wing", "wo", "wok", "wong", "wu", "wui", "wun", "wut", "za", "zaa",
    "zaai", "zaak", "zaam", "zaan", "zaang", "zaap", "zaat", "zaau", "zai", "zak", "zam", "zan",
    "zang", "zap", "zat", "zau", "ze", "zek", "zem", "zeng", "zeoi", "zeon", "zeot", "zep", "zet",
    "zeu", "zi", "zik", "zim", "zin", "zing", "zip", "zit", "ziu", "zo", "zoe", "zoek", "zoeng",
    "zoet", "zoi", "zok", "zong", "zou", "zuk", "zung", "zyu", "zyun", "zyut",
];

/// 切分上限，与普通话音节表一致。
const MAX_SEGMENTATIONS: usize = 8;

/// 每个位置最多留几种切法。
const MAX_PER_POSITION: usize = 8;

pub fn is_syllable(text: &str) -> bool {
    SYLLABLES.binary_search(&text).is_ok()
}

/// `text` 是不是某个合法粤拼音节的**真**前缀。
pub fn is_syllable_prefix(text: &str) -> bool {
    SYLLABLES
        .iter()
        .any(|s| s.len() > text.len() && s.starts_with(text))
}

/// 把一段无分隔的粤拼切成音节。允许 `'` 强制分隔（`nei'hou`），末尾允许一个没打完的音节。
/// 返回按「音节少、不完整音节少、前面的音节长」排序的切法，最多 [`MAX_SEGMENTATIONS`] 种。
pub fn segment(input: &str) -> Result<Vec<Segmentation>, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    for (position, ch) in input.chars().enumerate() {
        if !(ch.is_ascii_lowercase() || ch == '\'') {
            return Err(ParseError::InvalidChar {
                position: position + 1,
                ch,
            });
        }
    }
    let chunks: Vec<&str> = input.split('\'').filter(|c| !c.is_empty()).collect();
    if chunks.is_empty() {
        return Err(ParseError::NoSegmentation);
    }
    let mut results = vec![Segmentation {
        syllables: Vec::new(),
    }];
    for (index, chunk) in chunks.iter().enumerate() {
        let is_last = index + 1 == chunks.len();
        let options = segment_chunk(chunk, is_last);
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

fn segment_chunk(chunk: &str, allow_partial: bool) -> Vec<Segmentation> {
    let n = chunk.len();
    let mut best: Vec<Vec<Vec<Syllable>>> = vec![Vec::new(); n + 1];
    best[0].push(Vec::new());
    for start in 0..n {
        if best[start].is_empty() {
            continue;
        }
        let rest = &chunk[start..];
        for syllable in SYLLABLES.iter().copied().filter(|s| rest.starts_with(s)) {
            let end = start + syllable.len();
            let mut candidates = best[start].clone();
            for candidate in &mut candidates {
                candidate.push(Syllable::complete(syllable));
            }
            offer(&mut best[end], candidates);
        }
        if allow_partial && is_syllable_prefix(rest) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(segmentation: &Segmentation) -> Vec<(String, bool)> {
        segmentation
            .syllables
            .iter()
            .map(|s| (s.text.clone(), s.complete))
            .collect()
    }

    #[test]
    fn segments_jyutping_without_separators() {
        let result = segment("neihou").unwrap();
        assert_eq!(
            texts(&result[0]),
            [("nei".to_owned(), true), ("hou".to_owned(), true)]
        );
    }

    #[test]
    fn apostrophe_forces_a_boundary() {
        let result = segment("nei'hou").unwrap();
        assert_eq!(result[0].syllables.len(), 2);
        assert_eq!(segment("nei'hou").unwrap()[0].syllables[0].text, "nei");
    }

    #[test]
    fn trailing_partial_is_allowed() {
        let result = segment("neih").unwrap();
        let best = &result[0];
        assert_eq!(best.syllables.len(), 2);
        assert_eq!(best.syllables[1].text, "h");
        assert!(!best.syllables[1].complete);
    }

    #[test]
    fn full_syllables_are_not_partial() {
        let result = segment("hou").unwrap();
        assert_eq!(texts(&result[0]), [("hou".to_owned(), true)]);
    }

    #[test]
    fn unknown_input_is_rejected() {
        assert!(segment("nei").is_ok());
        // 粤拼没有 x 声母，`neix` 切不动
        assert!(segment("neix").is_err());
        assert!(segment("xyz").is_err());
    }
}
