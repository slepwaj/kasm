//! Xina 拼音：声调符号也是音节的一部分，按罗马字管线查词。

use kasm_dictionary::Dictionary;

use crate::Engine;

fn engine() -> Engine {
    let dictionary = Dictionary::parse("〇\tling@\t1\n零\tling@\t1\n").unwrap();
    let mut engine = Engine::new(dictionary);
    engine.set_xina(true);
    engine
}

#[test]
fn queries_a_toned_syllable() {
    let mut engine = engine();
    engine.set_input("ling@");
    let query = engine.query().unwrap();
    assert!(query.candidates.items.iter().any(|item| item.text == "〇"));
    assert_eq!(crate::xina::display_reading("ling@ ling#"), "líng lǐng");
}
