//! tupa：中古音罗马字，走与通字相同的罗马字管线。

use kasm_dictionary::Dictionary;

use crate::Engine;

const DICT: &str = "之\ttjy\t80000\n也\tjaq\t50000\n不\tpuq\t60000\n夫\tpa\t90000\n";

#[test]
fn tupa_queries_by_its_reading() {
    let mut engine = Engine::new(Dictionary::parse(DICT).unwrap());
    engine.set_tupa(true);
    engine.set_input("tjy");
    assert_eq!(engine.query().unwrap().candidates.items[0].text, "之");
    engine.set_input("puq");
    assert_eq!(engine.query().unwrap().candidates.items[0].text, "不");
}

#[test]
fn shanggu_queries_by_its_reading() {
    let mut engine = Engine::new(Dictionary::parse(DICT).unwrap());
    engine.set_shanggu(true);
    engine.set_input("pa");
    assert_eq!(engine.query().unwrap().candidates.items[0].text, "夫");
}
