//! 注音：大千键位先解成拼音，候选再按拼音查；声调暂时只保留在缓冲区，不参与筛选。

use kasm_dictionary::Dictionary;

use crate::Engine;

#[test]
fn decodes_dachen_keys_to_pinyin() {
    let dictionary = Dictionary::parse("你\tni\t1\n").unwrap();
    let mut engine = Engine::new(dictionary);
    engine.set_zhuyin_mode(true);
    engine.set_input("su ");
    let query = engine.query().unwrap();
    assert!(query.candidates.items.iter().any(|item| item.text == "你"));
    assert!(query.marked_text().contains("ㄋㄧ"));
}
