//! Xina 传统派普通话拼音：无调拼式走罗马字查询。

use kasm_dictionary::Dictionary;

use crate::Engine;

fn engine() -> Engine {
    let dictionary = Dictionary::parse("零\tling\t1\n").unwrap();
    let mut engine = Engine::new(dictionary);
    engine.set_xina(true);
    engine
}

#[test]
fn queries_a_xina_syllable() {
    let mut engine = engine();
    engine.set_input("ling");
    let query = engine.query().unwrap();
    assert!(query.candidates.items.iter().any(|item| item.text == "零"));
    assert_eq!(crate::xina::display_reading("ling"), "ling");
}
