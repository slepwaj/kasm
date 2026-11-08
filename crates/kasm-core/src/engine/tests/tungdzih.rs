//! 通字查询：音节切分、候选、整句拼词。

use kasm_dictionary::Dictionary;

use crate::{CandidateKind, Engine};

fn tungdzih() -> Engine {
    let dict = "你\tnii\t80000\n好\txao\t30000\n我\tqoo\t90000\n乌\tu\t100\n鸦\tea\t100\n";
    let mut engine = Engine::new(Dictionary::parse(dict).unwrap());
    engine.set_tungdzih(true);
    engine
}

#[test]
fn queries_tungdzih_by_romanization() {
    let mut engine = tungdzih();
    engine.set_input("nii");
    let query = engine.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "你");
    assert_eq!(query.candidates.items[0].reading.as_deref(), Some("nii"));
}

#[test]
fn composes_words_from_characters() {
    // 通字词库以单字为主，`niixao` 由「你 + 好」拼成整句候选
    let mut engine = tungdzih();
    engine.set_input("niixao");
    let query = engine.query().unwrap();
    let first = &query.candidates.items[0];
    assert_eq!(first.kind, CandidateKind::Sentence);
    assert_eq!(first.text, "你好");
    assert_eq!(first.reading.as_deref(), Some("nii xao"));
}

#[test]
fn commit_consumes_every_letter() {
    let mut engine = tungdzih();
    engine.set_input("niixao");
    let candidate = engine.query().unwrap().candidates.items[0].clone();
    assert_eq!(engine.commit(&candidate), "你好");
    assert!(engine.composition().text().is_empty());
}

#[test]
fn single_letter_syllable_and_two_chars() {
    // `u` 是合法音节（乌），`uea` = u + ea（乌鸦）；这两个以前被「问字模式 u」抢走
    let mut engine = tungdzih();
    engine.set_input("u");
    let query = engine.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "乌");

    engine.set_input("uea");
    let query = engine.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "乌鸦");
    assert_eq!(query.candidates.items[0].reading.as_deref(), Some("u ea"));
}
