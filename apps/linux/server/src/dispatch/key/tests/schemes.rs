//! 双拼模式入口与提交键边界。
use super::support::{compose, key, router};
use kasm_core::{ModeKeys, ShuangpinScheme};
use kasm_platform::protocol::{KeyModifiers, KeyOutcome};

#[test]
fn shuangpin_shift_mode_keys_respect_custom_bindings_and_english() {
    let mut router = router(5);
    let normal = KeyModifiers::default();
    router.engine.set_shuangpin(Some(ShuangpinScheme::Xiaohe));
    compose(&mut router, "V1+2", normal);
    assert!(router.engine.expression_mode());
    assert_eq!(
        key(&mut router, 32, Some(' '), normal).1.as_deref(),
        Some("3")
    );
    compose(&mut router, "U4e00", normal);
    assert!(router.engine.question_mode());
    assert_eq!(
        key(&mut router, 32, Some(' '), normal).1.as_deref(),
        Some("一")
    );
    router.engine.set_mode_keys(ModeKeys {
        expression: 'i',
        question: 'v',
        question_mark: false,
    });
    compose(&mut router, "I1+2", normal);
    assert!(router.engine.expression_mode());
    key(&mut router, 27, None, normal);
    let english = KeyModifiers {
        english_mode: true,
        ..normal
    };
    compose(&mut router, "I", english);
    assert!(router.engine.english_mode());
    assert_eq!(key(&mut router, 13, None, english).1.as_deref(), Some("I"));
    key(&mut router, 27, None, english);
    router.engine.set_shuangpin(None);
    assert_eq!(
        key(&mut router, b'I' as u32, Some('I'), normal).0,
        KeyOutcome::Passthrough
    );
}
