//! Ctrl 在中文与英文显示优先间切换，以及两栏候选的上屏行为。

use crate::composing::SentenceTranslator;
use crate::support::*;
use qingjian_render::HitTarget;

fn toggle(router: &mut Router) -> (KeyOutcome, Option<String>, Frame) {
    press(router, function_key(KeyEvent::SWAP_TRANSLATION))
}

fn poll(router: &mut Router) -> (Frame, Option<String>) {
    match router.handle(ClientMessage::Poll { session: SESSION }) {
        Some(ServerMessage::Update { frame, commit, .. }) => (frame, commit),
        other => panic!("expected Update, got {other:?}"),
    }
}

#[test]
fn ctrl_swaps_candidate_columns_and_back_without_changing_pinyin() {
    let mut router = router();
    let (_, _, chinese) = type_letters(&mut router, "nihao");
    assert_eq!(preedit(&chinese), "ni'hao");
    assert_eq!(chinese.candidates.items[0].text, "你好");

    let (outcome, commit, english) = toggle(&mut router);
    assert_eq!((outcome, commit), (KeyOutcome::Consumed, None));
    assert_eq!(preedit(&english), "ni'hao");
    let hello = english
        .candidates
        .items
        .iter()
        .find(|candidate| candidate.text == "hello")
        .expect("英文栏应显示 hello");
    assert_eq!(hello.translation.as_ref().unwrap().senses()[0].text, "你好");

    let (_, _, restored) = toggle(&mut router);
    assert_eq!(preedit(&restored), "ni'hao");
    assert!(
        restored
            .candidates
            .items
            .iter()
            .any(|candidate| candidate.text == "你好")
    );
}

#[test]
fn ctrl_then_space_commits_translation_and_starts_a_chinese_composition() {
    let mut router = router();
    type_letters(&mut router, "nihao");
    toggle(&mut router);
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("hello"));
    assert!(after.is_empty());

    let (_, _, next) = type_letters(&mut router, "nihao");
    assert_eq!(next.candidates.items[0].text, "你好");
}

#[test]
fn highlighted_nonfirst_translation_commits_its_english_text() {
    let mut router = router();
    let (_, _, frame) = type_letters(&mut router, "shi");
    let index = frame
        .candidates
        .items
        .iter()
        .enumerate()
        .find(|(index, candidate)| *index > 0 && candidate.translation.is_some())
        .map(|(index, _)| index)
        .expect("shi 应有非首位的带译文候选");
    for _ in 0..index {
        press(&mut router, function_key(0x28));
    }
    let highlighted = router.handle(ClientMessage::Key {
        session: SESSION,
        event: function_key(KeyEvent::SWAP_TRANSLATION),
    });
    let (_, _, frame) = key_result(highlighted);
    assert_eq!(frame.highlight, index);
    let expected = frame.candidates.items[index].text.clone();
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some(expected.as_str()));
    assert!(after.is_empty());
}

#[test]
fn page_selection_commits_translation_of_the_page_candidate() {
    let mut router = router_with(RouterConfig {
        page_size: 1,
        ..RouterConfig::default()
    });
    let (_, _, frame) = type_letters(&mut router, "shi");
    assert!(frame.page_count > 1);
    let (_, _, mut page) = press(&mut router, function_key(0x22));
    assert_eq!(page.page, 1, "Page Down 应先翻到第二页");
    let mut expected = None;
    for _ in page.page..page.page_count {
        if let Some(translation) = page.candidates.items[0].translation.as_ref() {
            expected = Some(translation.senses()[0].text.clone());
            break;
        }
        if page.page + 1 < page.page_count {
            let (_, _, next) = press(&mut router, function_key(0x22));
            page = next;
        }
    }
    let expected = expected.expect("shi 的非首页候选「时」「事」「十」在样例释义表中有译文");
    assert!(page.page > 0);
    toggle(&mut router);
    let (_, commit, after) = press(&mut router, digit(1));
    assert_eq!(commit.as_deref(), Some(expected.as_str()));
    assert!(after.is_empty());
}

#[test]
fn clicking_the_main_column_commits_english_after_ctrl() {
    let mut router = router();
    type_letters(&mut router, "nihao");
    toggle(&mut router);
    router.handle_click(HitTarget::Candidate(0));
    let (after, commit) = poll(&mut router);
    assert_eq!(commit.as_deref(), Some("hello"));
    assert!(after.is_empty());
}

#[test]
fn clicking_the_annotation_column_commits_chinese_after_ctrl() {
    let mut router = router();
    type_letters(&mut router, "nihao");
    toggle(&mut router);
    router.handle_click(HitTarget::Translation {
        candidate: 0,
        sense: 0,
    });
    let (after, commit) = poll(&mut router);
    assert_eq!(commit.as_deref(), Some("你好"));
    assert!(after.is_empty());
}

#[test]
fn disabled_ctrl_swap_does_not_change_the_candidate_view() {
    let mut router = router_with(RouterConfig {
        ctrl_swap_translation: false,
        ..RouterConfig::default()
    });
    let (_, _, before) = type_letters(&mut router, "nihao");
    let (outcome, commit, after) = toggle(&mut router);
    assert_eq!((outcome, commit), (KeyOutcome::Passthrough, None));
    assert_eq!(candidate_texts(&after), candidate_texts(&before));
}

#[test]
fn ctrl_without_a_composition_does_not_toggle_translation() {
    let mut router = router();
    let (outcome, commit, empty) = toggle(&mut router);
    assert_eq!((outcome, commit), (KeyOutcome::Passthrough, None));
    assert!(empty.is_empty());
    let (_, _, frame) = type_letters(&mut router, "nihao");
    assert_eq!(frame.candidates.items[0].text, "你好");
}

#[test]
fn persistent_english_mode_does_not_toggle_translation() {
    let mut router = router_with(RouterConfig {
        default_english: true,
        ..RouterConfig::default()
    });
    type_english(&mut router, "hello");
    let (outcome, commit, frame) = toggle(&mut router);
    assert_eq!((outcome, commit), (KeyOutcome::Passthrough, None));
    assert_eq!(preedit(&frame), "hello");
}

#[test]
fn candidate_without_translation_keeps_composition_until_ctrl_restores_chinese() {
    let mut router = router();
    let (_, _, frame) = type_letters(&mut router, "fa");
    let index = frame
        .candidates
        .items
        .iter()
        .position(|candidate| candidate.translation.is_none())
        .expect("fa 应有无译文候选");
    for _ in 0..index {
        press(&mut router, function_key(0x28));
    }
    toggle(&mut router);
    let (_, commit, waiting) = press(&mut router, punct(' '));
    assert_eq!(commit, None);
    assert_eq!(preedit(&waiting), "fa");
    assert!(!waiting.is_empty());

    toggle(&mut router);
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(
        commit.as_deref(),
        Some(frame.candidates.items[index].text.as_str())
    );
    assert!(after.is_empty());
}

#[test]
fn ctrl_c_passes_through_without_toggling_translation() {
    let mut router = router();
    let (_, _, before) = type_letters(&mut router, "nihao");
    let ctrl_c = letter_with(
        'c',
        KeyModifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let (outcome, commit, after) = press(&mut router, ctrl_c);
    assert_eq!((outcome, commit), (KeyOutcome::Passthrough, None));
    assert_eq!(preedit(&after), preedit(&before));
    assert_eq!(candidate_texts(&after), candidate_texts(&before));
}

#[test]
fn translated_sentence_commits_its_full_english_text() {
    let submitted = Arc::new(Mutex::new(Vec::new()));
    let mut router = router_with(RouterConfig {
        sentence_translation: true,
        ..RouterConfig::default()
    });
    router
        .engine_mut()
        .set_predictor(Box::new(SentenceTranslator::new(submitted)));
    let (_, _, frame) = type_letters(&mut router, "xiangkaifa");
    let sentence = frame
        .candidates
        .items
        .iter()
        .find(|candidate| candidate.text == "想开发")
        .expect("样例词库应有整句候选「想开发」");
    assert_eq!(
        sentence.translation.as_ref().unwrap().senses()[0].text,
        "It seems not to work."
    );
    toggle(&mut router);
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("It seems not to work."));
    assert!(after.is_empty());
}

#[test]
fn physical_ctrl_down_never_swaps_even_before_modifier_state_updates() {
    let mut router = router();
    let (_, _, before) = type_letters(&mut router, "nihao");
    for modifiers in [KeyModifiers::default(), CTRL] {
        let (outcome, commit, after) = press(&mut router, KeyEvent::new(0x11, None, modifiers));
        assert_eq!((outcome, commit), (KeyOutcome::Passthrough, None));
        assert_eq!(candidate_texts(&after), candidate_texts(&before));
    }
    let (_, _, after) = toggle(&mut router);
    assert_eq!(after.candidates.items[0].text, "hello");
}
