//! Xina 切拼：中古音单字表，按音节直接查词。

use kasm_dictionary::Dictionary;

use crate::Engine;

#[test]
fn queries_a_qiepin_syllable() {
    let dictionary = Dictionary::parse("矮\tqeeq\t1\n").unwrap();
    let mut engine = Engine::new(dictionary);
    engine.set_xina_qiepin(true);
    engine.set_input("qeeq");
    let query = engine.query().unwrap();
    assert!(query.candidates.items.iter().any(|item| item.text == "矮"));
}
