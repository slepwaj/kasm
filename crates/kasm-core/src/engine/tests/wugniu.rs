//! 吳語：去掉声调数字的苏州吴语罗马字，按音节直接查词。

use kasm_dictionary::Dictionary;

use crate::Engine;

#[test]
fn queries_a_wugniu_syllable() {
    let dictionary = Dictionary::parse("資\ttsy\t1\n").unwrap();
    let mut engine = Engine::new(dictionary);
    engine.set_wugniu(true);
    engine.set_input("tsy");
    let query = engine.query().unwrap();
    assert!(query.candidates.items.iter().any(|item| item.text == "資"));
}
