//! 通字（Tungdzih / General Chinese）：赵元任《通字方案》的罗马字音节表与切分。
//!
//! 与粤拼一样是独立管线：音节表、切分、查询各走一条旁路，不进普通话音节表。
//! 音节表由 `data/tungdzih-syllables.txt` 编译进二进制（`tools/rebuild-tungdzih.py`
//! 从《通字_合并.xlsx》重建；更早来自 `参考项目/tungdzih` 的 `tungdzih.dict.yaml`）；
//! 拼式本身带声调字母（`你 nii`、`好 xao/xaw`），与通字方案一致。

use std::sync::LazyLock;

use crate::parser::{ParseError, Segmentation};
use crate::romanization::Table;

static TABLE: LazyLock<Table> =
    LazyLock::new(|| Table::from_source(include_str!("../../data/tungdzih-syllables.txt")));

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
    fn segments_tungdzih_without_separators() {
        // 你 nii + 好 xao
        let result = segment("niixao").unwrap();
        assert_eq!(
            texts(&result[0]),
            [("nii".to_owned(), true), ("xao".to_owned(), true)]
        );
    }

    #[test]
    fn apostrophe_forces_a_boundary() {
        let result = segment("nii'xao").unwrap();
        assert_eq!(result[0].syllables.len(), 2);
        assert_eq!(result[0].syllables[0].text, "nii");
    }

    #[test]
    fn trailing_partial_is_allowed() {
        let result = segment("niix").unwrap();
        let best = &result[0];
        assert_eq!(best.syllables.len(), 2);
        assert_eq!(best.syllables[1].text, "x");
        assert!(!best.syllables[1].complete);
    }

    #[test]
    fn unknown_input_is_rejected() {
        assert!(segment("nii").is_ok());
        assert!(segment("123").is_err());
    }
}
