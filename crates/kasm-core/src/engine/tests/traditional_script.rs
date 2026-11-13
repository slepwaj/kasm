//! 通字简 / 繁两套词库的整份切换。

use std::collections::HashMap;

use kasm_dictionary::Dictionary;

use crate::{Engine, TraditionalVariant};

fn engine() -> Engine {
    let simplified = Dictionary::parse("什么\tzhip me\t100\n谢谢\tszieh szieh\t100\n").unwrap();
    let traditional = Dictionary::parse("什麼\tzhip me\t100\n謝謝\tszieh szieh\t100\n").unwrap();
    let mut engine = Engine::new(simplified);
    engine.set_tungdzih(true);
    engine.set_traditional_dictionary(traditional);
    engine.set_traditional_seed(HashMap::from([
        ("什麼".to_owned(), "什么".to_owned()),
        ("謝謝".to_owned(), "谢谢".to_owned()),
    ]));
    engine
}

fn first(engine: &mut Engine, keys: &str) -> String {
    engine.set_input(keys);
    engine.query().unwrap().candidates.items[0].text.clone()
}

#[test]
fn switching_traditional_swaps_the_whole_dictionary() {
    let mut engine = engine();
    assert_eq!(first(&mut engine, "zhipme"), "什么");

    engine.set_traditional(TraditionalVariant::Taiwan);
    assert_eq!(first(&mut engine, "zhipme"), "什麼");

    engine.set_traditional(TraditionalVariant::Off);
    assert_eq!(first(&mut engine, "zhipme"), "什么");
}
