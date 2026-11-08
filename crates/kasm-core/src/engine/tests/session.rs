//! 会话状态的挂起与丢弃：每个输入上下文各一份，切走再切回不能串。

use kasm_dictionary::AuxCodeTable;

use super::*;

/// 只覆盖样例词库里 `kaifa` 能查出来的那几条（与 `aux_code` 测试同一张表）。
fn codes() -> Arc<dyn AuxCodeLookup> {
    Arc::new(
        AuxCodeTable::from_pairs([
            ("开发".to_owned(), "kf".to_owned()),
            ("开发者".to_owned(), "kfz".to_owned()),
            ("开".to_owned(), "kh".to_owned()),
        ])
        .unwrap(),
    )
}

/// 进辅码态并敲下一段码。
fn engine_in_aux() -> Engine {
    let mut engine = engine().with_aux_codes(vec![codes()]);
    engine.set_aux_enabled(true);
    engine.set_input("kaifa");
    assert!(engine.aux_trigger(';'));
    engine.enter_aux();
    assert!(engine.push_aux_code('k'));
    engine
}

#[test]
fn aux_code_travels_with_the_session() {
    let mut engine = engine_in_aux();
    assert!(engine.in_aux());
    assert_eq!(engine.aux_code(), "k");

    let mut session = EngineSession::default();
    engine.swap_session(&mut session);
    // 换进来的是另一个上下文的空白状态，码段不能漏过来
    assert!(!engine.in_aux());
    assert_eq!(engine.aux_code(), "");

    engine.swap_session(&mut session);
    // 切回去，码段原样接上
    assert!(engine.in_aux());
    assert_eq!(engine.aux_code(), "k");
}

#[test]
fn discarding_input_drops_the_aux_code() {
    let mut engine = engine_in_aux();
    engine.discard_input();
    assert!(!engine.in_aux());
    assert_eq!(engine.aux_code(), "");

    // 挂起中的上下文同样清掉：隐私边界丢的是这份状态，不是当前装着的这份
    let mut engine = engine_in_aux();
    let mut session = EngineSession::default();
    engine.swap_session(&mut session);
    session.discard_input();
    engine.swap_session(&mut session);
    assert!(!engine.in_aux());
    assert_eq!(engine.aux_code(), "");
}
