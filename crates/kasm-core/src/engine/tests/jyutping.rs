//! 粤拼方案：无调 Jyutping 的切分、前缀查词与排序。

use super::*;

const CANTONESE: &str = "你好\tnei hou\t9000\n我\tngo\t8000\n佢\tkeoi\t7000\n唔該\tm goi\t6000\n早晨\tzou san\t5000\n去\theoi\t4000\n";

fn cantonese() -> Engine {
    let mut engine = Engine::new(Dictionary::parse(CANTONESE).unwrap());
    engine.set_jyutping(true);
    engine
}

fn texts(engine: &Engine) -> Vec<String> {
    engine
        .query()
        .unwrap()
        .candidates
        .items
        .into_iter()
        .map(|candidate| candidate.text)
        .collect()
}

#[test]
fn queries_cantonese_by_jyutping() {
    let mut engine = cantonese();
    engine.set_input("neihou");
    assert_eq!(texts(&engine)[0], "你好");

    engine.set_input("ngo");
    assert_eq!(texts(&engine)[0], "我");

    engine.set_input("mgoi");
    assert_eq!(texts(&engine)[0], "唔該");
}

#[test]
fn apostrophe_and_partial_input() {
    let mut engine = cantonese();
    engine.set_input("nei'hou");
    assert_eq!(texts(&engine)[0], "你好");

    engine.set_input("neih");
    // 末尾 `h` 是没打完的音节，前缀查词仍能命中「你好」
    assert_eq!(texts(&engine)[0], "你好");
}

#[test]
fn jyutping_keeps_pinyin_syllables_out() {
    // `nihao` 是普通拼音，不是粤拼（`ni` / `hao` 不在粤拼音节表里）；粤拼模式下应切不动
    let mut engine = cantonese();
    engine.set_input("nihao");
    // 切不动时退回原样英文（与粤拼共用一条兜底），但不会出中文候选
    if let Ok(query) = engine.query() {
        assert!(
            query
                .candidates
                .items
                .iter()
                .all(|candidate| candidate.kind != CandidateKind::Chinese)
        );
    }
}

#[test]
fn sentence_candidate_comes_first() {
    // 词图整句：词级没有覆盖整段的单个词时，把多个词拼成一句放在最前（[句]）
    let dict = "我\tngo\t9000\n哋\tdei\t8000\n去\theoi\t7000\n我哋\tngo dei\t8500\n";
    let mut engine = Engine::new(Dictionary::parse(dict).unwrap());
    engine.set_jyutping(true);
    engine.set_input("ngodeiheoi");
    let query = engine.query().unwrap();
    let first = &query.candidates.items[0];
    assert_eq!(first.kind, CandidateKind::Sentence, "首个候选应是整句");
    assert_eq!(first.text, "我哋去");
    assert_eq!(first.syllables, ["ngo", "dei", "heoi"]);
    assert_eq!(first.reading.as_deref(), Some("ngo dei heoi"));
}

#[test]
fn every_candidate_carries_its_jyutping() {
    // 候选窗右侧的 annotation：每个候选都带自己的无调粤拼，与敲法一致
    let mut engine = cantonese();
    engine.set_input("neihou");
    let items = engine.query().unwrap().candidates.items;
    assert_eq!(items[0].text, "你好");
    assert_eq!(items[0].reading.as_deref(), Some("nei hou"));
    assert!(
        items.iter().all(|item| item
            .reading
            .as_deref()
            .is_some_and(|reading| !reading.is_empty())),
        "每个候选都应带粤拼"
    );
}

#[test]
fn commit_consumes_every_letter() {
    // 回归：普通话音节表的「一处编辑」纠错曾把 `keoi` 纠成 `kei`，上屏只吃 3 个字母，
    // 剩下的 `i` 又开一段组句（TSF 里表现为「佢i」）。
    let mut engine = cantonese();
    engine.set_input("keoi");
    let candidate = engine
        .query()
        .unwrap()
        .candidates
        .items
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(candidate.text, "佢");
    assert_eq!(engine.commit(&candidate), "佢");
    assert!(
        engine.composition().text().is_empty(),
        "上屏后不应留下残余字母；composition = {:?}",
        engine.composition().text()
    );
}
