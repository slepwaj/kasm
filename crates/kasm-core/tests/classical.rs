//! 古籍字頻統計的两套罗马字方案查询冒烟测试。

use kasm_core::Engine;
use kasm_dictionary::Dictionary;

fn engine(path: &str) -> Engine {
    Engine::new(Dictionary::from_path(path).expect("词库读取失败"))
}

#[test]
fn unt_queries_hanzi() {
    let mut engine = engine("../../assets/lexicon/unt.tsv");
    engine.set_unt(true);
    engine.set_input("tɕɨ");
    let query = engine.query().expect("查询失败");
    assert!(
        query.candidates.items.iter().any(|c| c.text == "之"),
        "候选：{:?}",
        query
            .candidates
            .items
            .iter()
            .map(|c| &c.text)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gcr_queries_hanzi() {
    let mut engine = engine("../../assets/lexicon/gcr.tsv");
    engine.set_gcr(true);
    engine.set_input("çi");
    let query = engine.query().expect("查询失败");
    assert!(
        query.candidates.items.iter().any(|c| c.text == "之"),
        "候选：{:?}",
        query
            .candidates
            .items
            .iter()
            .map(|c| &c.text)
            .collect::<Vec<_>>()
    );
}

#[test]
fn gcr_partial_commit_does_not_panic() {
    let mut engine = engine("../../assets/lexicon/gcr.tsv");
    engine.set_gcr(true);
    engine.set_input("ç");
    let query = engine.query().expect("查询失败");
    let candidate = query.candidates.items.first().expect("应有候选").clone();
    let committed = engine.commit(&candidate);
    assert!(!committed.is_empty(), "部分输入也要能上屏");
}

#[test]
fn unt_partial_commit_does_not_panic() {
    let mut engine = engine("../../assets/lexicon/unt.tsv");
    engine.set_unt(true);
    engine.set_input("tɕ");
    let query = engine.query().expect("查询失败");
    let candidate = query.candidates.items.first().expect("应有候选").clone();
    let committed = engine.commit(&candidate);
    assert!(!committed.is_empty(), "部分输入也要能上屏");
}

#[test]
fn gcr_sentence_without_tones() {
    let mut engine = engine("../../assets/lexicon/gcr.tsv");
    engine.set_gcr(true);
    for (reading, expected) in [
        ("ţyenq", "轉"),
        ("toꝩh", "到"),
        ("liq", "裡"),
        ("khiwq", "去"),
        ("ȝuojh", "會"),
        ("rioŋh", "讓"),
        ("pịk", "逼"),
        ("rin", "人"),
        ("no", "哪"),
        ("fuŋ", "瘋"),
    ] {
        engine.set_input(reading);
        let query = engine.query().expect("查询失败");
        assert!(
            query.candidates.items.iter().any(|c| c.text == expected),
            "{reading} 没有 {expected}：{:?}",
            query.candidates.items.iter().map(|c| &c.text).collect::<Vec<_>>()
        );
    }
}
