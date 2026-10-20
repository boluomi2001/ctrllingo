//! Ctrl 按需翻译的请求次数、当前候选绑定、取消与失败重试。

use std::collections::VecDeque;

use crate::support::*;
use qingjian_core::{
    CandidateKind, CloudWord, Prediction, PredictionKind, PredictionPolicy, PredictionRequest,
    Predictor,
};

#[derive(Clone, Default)]
struct ControlledPredictor {
    submitted: Arc<Mutex<Vec<PredictionRequest>>>,

    replies: Arc<Mutex<VecDeque<Prediction>>>,

    automatic: bool,
}

impl Predictor for ControlledPredictor {
    fn policy(&self) -> PredictionPolicy {
        PredictionPolicy {
            automatic: self.automatic,
            slots: 0,
            max_items: 0,
            sentence: self.automatic,
            ..PredictionPolicy::default()
        }
    }

    fn submit(&mut self, request: PredictionRequest) {
        self.submitted.lock().unwrap().push(request);
    }

    fn poll(&mut self) -> Option<Prediction> {
        self.replies.lock().unwrap().pop_front()
    }
}

fn controlled_router() -> (Router, ControlledPredictor) {
    controlled_router_for_language(Language::English)
}

fn controlled_router_for_language(language: Language) -> (Router, ControlledPredictor) {
    let predictor = ControlledPredictor::default();
    let mut router = router_for_language(
        RouterConfig {
            cloud_enabled: false,
            cloud_slots: 0,
            sentence_translation: true,
            ..RouterConfig::default()
        },
        language,
    );
    router
        .engine_mut()
        .set_predictor(Box::new(predictor.clone()));
    (router, predictor)
}

fn automatic_router_for_language(language: Language) -> (Router, ControlledPredictor) {
    let predictor = ControlledPredictor {
        automatic: true,
        ..ControlledPredictor::default()
    };
    let mut router = router_for_language(
        RouterConfig {
            cloud_enabled: true,
            cloud_slots: 0,
            sentence_translation: true,
            ..RouterConfig::default()
        },
        language,
    );
    router
        .engine_mut()
        .set_predictor(Box::new(predictor.clone()));
    (router, predictor)
}

#[test]
fn non_english_languages_request_and_display_the_matching_sentence_translation() {
    for (language, code) in [
        (Language::Japanese, "ja"),
        (Language::Spanish, "es"),
        (Language::Vietnamese, "vi"),
        (Language::German, "de"),
    ] {
        let (mut router, predictor) = controlled_router_for_language(language);
        type_letters(&mut router, "xiangkaifa");
        toggle(&mut router);
        let request = translations(&predictor).pop().expect("Ctrl 应请求整句翻译");
        assert_eq!(request.target_language, code);
        let (_, commit, waiting) = press(&mut router, punct(' '));
        assert_eq!(commit, None);
        assert_eq!(preedit(&waiting), "xiang'kai'fa");
        queue_translation(&predictor, &request, Some("translated sentence"));
        let (frame, _) = poll(&mut router);
        assert!(candidate_texts(&frame).contains(&"translated sentence"));
        let (_, commit, after) = press(&mut router, punct(' '));
        assert_eq!(commit.as_deref(), Some("translated sentence"));
        assert!(after.is_empty());
    }
}

#[test]
fn old_compose_reply_and_surrounding_refresh_do_not_replace_a_pending_translation() {
    let (mut router, predictor) = automatic_router_for_language(Language::English);
    type_letters(&mut router, "xiangkaifa");
    let compose = requests(&predictor, PredictionKind::Compose)
        .pop()
        .expect("输入期间应提交自动组句请求");
    toggle(&mut router);
    let translate = translations(&predictor).pop().expect("Ctrl 应提交翻译请求");
    let compose_count = requests(&predictor, PredictionKind::Compose).len();

    router.handle(ClientMessage::Surrounding {
        session: SESSION,
        text: "前文刷新".to_owned(),
    });
    assert_eq!(
        requests(&predictor, PredictionKind::Compose).len(),
        compose_count
    );

    queue(
        &predictor,
        Prediction {
            sequence: compose.sequence,
            words: vec![CloudWord {
                text: "想".to_owned(),
                syllables: vec!["xiang".to_owned()],
                reading: None,
            }],
            sentence: Some("旧组句结果".to_owned()),
        },
    );
    queue_translation(&predictor, &translate, Some("the requested translation"));
    let (compose_frame, _) = poll(&mut router);
    assert!(!candidate_texts(&compose_frame).contains(&"旧组句结果"));
    let (translated_frame, _) = poll(&mut router);
    assert!(candidate_texts(&translated_frame).contains(&"the requested translation"));
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("the requested translation"));
    assert!(after.is_empty());
}

#[test]
fn manual_prediction_during_translation_wait_returns_to_chinese_and_requests_compose() {
    let (mut router, predictor) = automatic_router_for_language(Language::English);
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    assert_eq!(translations(&predictor).len(), 1);
    let compose_count = requests(&predictor, PredictionKind::Compose).len();

    let predict_key = KeyEvent::new(
        0x4A,
        Some('j'),
        KeyModifiers {
            ctrl: true,
            alt: true,
            ..Default::default()
        },
    );
    let frame = press(&mut router, predict_key).2;
    assert!(candidate_texts(&frame).contains(&"想开发"));
    assert_eq!(translations(&predictor).len(), 1);
    let compose = requests(&predictor, PredictionKind::Compose);
    assert_eq!(compose.len(), compose_count + 1);
    assert!(compose.last().unwrap().manual);

    queue(
        &predictor,
        Prediction {
            sequence: compose.last().unwrap().sequence,
            words: Vec::new(),
            sentence: Some("manual compose sentence".to_owned()),
        },
    );
    let (after, _) = poll(&mut router);
    assert!(candidate_texts(&after).contains(&"想开发"));
    assert!(!candidate_texts(&after).contains(&"manual compose sentence"));
}

#[test]
fn late_japanese_translation_is_discarded_after_switching_to_german() {
    let (mut router, predictor) = controlled_router_for_language(Language::Japanese);
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    let japanese = translations(&predictor).pop().expect("应提交日语翻译请求");
    assert_eq!(japanese.target_language, "ja");

    let german = qingjian_translate::Glossary::empty(Language::German);
    router.engine_mut().set_translator(Box::new(german));
    queue_translation(&predictor, &japanese, Some("日本語の古い結果"));
    let (after, commit) = poll(&mut router);
    assert_eq!(commit, None);
    assert!(candidate_texts(&after).contains(&"想开发"));
    assert!(!candidate_texts(&after).contains(&"日本語の古い結果"));

    toggle(&mut router);
    let requests = translations(&predictor);
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[1].target_language, "de");
}

fn translations(predictor: &ControlledPredictor) -> Vec<PredictionRequest> {
    predictor
        .submitted
        .lock()
        .unwrap()
        .iter()
        .filter(|request| request.kind == PredictionKind::Translate)
        .cloned()
        .collect()
}

fn requests(predictor: &ControlledPredictor, kind: PredictionKind) -> Vec<PredictionRequest> {
    predictor
        .submitted
        .lock()
        .unwrap()
        .iter()
        .filter(|request| request.kind == kind)
        .cloned()
        .collect()
}

fn queue(predictor: &ControlledPredictor, prediction: Prediction) {
    predictor.replies.lock().unwrap().push_back(prediction);
}

fn queue_translation(
    predictor: &ControlledPredictor,
    request: &PredictionRequest,
    text: Option<&str>,
) {
    predictor.replies.lock().unwrap().push_back(Prediction {
        sequence: request.sequence,
        words: Vec::new(),
        sentence: text.map(str::to_owned),
    });
}

fn poll(router: &mut Router) -> (Frame, Option<String>) {
    match router.handle(ClientMessage::Poll { session: SESSION }) {
        Some(ServerMessage::Update { frame, commit, .. }) => (frame, commit),
        other => panic!("expected Update, got {other:?}"),
    }
}

fn toggle(router: &mut Router) -> Frame {
    press(router, function_key(KeyEvent::SWAP_TRANSLATION)).2
}

#[test]
fn typing_stopping_and_navigation_never_request_a_sentence_translation() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    press(&mut router, function_key(0x28));
    press(&mut router, function_key(0x22));
    assert!(translations(&predictor).is_empty());

    press(&mut router, punct(' '));
    assert!(translations(&predictor).is_empty());
}

#[test]
fn ctrl_requests_only_the_highlighted_candidate_without_local_english() {
    let (mut router, predictor) = controlled_router();
    let (_, _, frame) = type_letters(&mut router, "xiangkaifa");
    let index = frame
        .candidates
        .items
        .iter()
        .enumerate()
        .find(|(index, candidate)| *index > 0 && candidate.translation.is_none())
        .map(|(index, _)| index)
        .expect("应有非首位且没有本地译文的候选");
    let mut selected_frame = frame.clone();
    for _ in 0..index {
        selected_frame = press(&mut router, function_key(0x28)).2;
    }
    let selected = selected_frame.candidates.items[selected_frame.highlight]
        .text
        .clone();
    toggle(&mut router);

    let requests = translations(&predictor);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].text, selected);
    assert!(requests[0].manual, "Ctrl 应走显式请求，跳过自动防抖");
}

#[test]
fn local_english_translation_swaps_without_a_network_request() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "nihao");
    let english = toggle(&mut router);
    assert!(candidate_texts(&english).contains(&"hello"));
    assert!(translations(&predictor).is_empty());
}

#[test]
fn changing_highlight_after_a_local_translation_returns_to_chinese_without_requesting() {
    let (mut router, predictor) = controlled_router();
    let (_, _, chinese) = type_letters(&mut router, "nihao");
    assert!(chinese.candidates.items.len() > 1);
    let english = toggle(&mut router);
    assert!(candidate_texts(&english).contains(&"hello"));

    let (_, _, after) = press(&mut router, function_key(0x28));
    assert_eq!(after.highlight, 1);
    assert_eq!(after.candidates.items[0].text, "你好");
    assert!(translations(&predictor).is_empty());
}

#[test]
fn ctrl_cancels_pending_intent_and_late_reply_cannot_change_chinese_view() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    let request = translations(&predictor).pop().expect("Ctrl 应触发一次请求");

    let chinese = toggle(&mut router);
    assert!(!chinese.candidates.items.iter().any(|candidate| {
        candidate.translation.as_ref().is_some_and(|translation| {
            translation
                .senses()
                .first()
                .is_some_and(|sense| sense.text == "Late sentence.")
        })
    }));
    queue_translation(&predictor, &request, Some("Late sentence."));
    let (after, commit) = poll(&mut router);
    assert_eq!(commit, None);
    assert_eq!(preedit(&after), "xiang'kai'fa");
    assert!(
        !after
            .candidates
            .items
            .iter()
            .any(|candidate| candidate.text == "Late sentence.")
    );
    assert_eq!(translations(&predictor).len(), 1);
}

#[test]
fn editing_cancels_the_translation_and_does_not_reissue_it() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    let old = translations(&predictor).pop().unwrap();

    press(&mut router, letter('z'));
    queue_translation(&predictor, &old, Some("Stale sentence."));
    let (after, _) = poll(&mut router);
    assert!(preedit(&after).ends_with('z'));
    assert!(
        !after
            .candidates
            .items
            .iter()
            .any(|candidate| candidate.text == "Stale sentence.")
    );
    assert!(
        translations(&predictor)
            .iter()
            .all(|request| request.sequence == old.sequence)
    );
}

#[test]
fn changing_highlight_returns_to_chinese_and_expires_the_old_translation() {
    let (mut router, predictor) = controlled_router();
    let (_, _, frame) = type_letters(&mut router, "xiangkaifa");
    let index = frame
        .candidates
        .items
        .iter()
        .enumerate()
        .find(|(index, candidate)| *index > 0 && candidate.translation.is_none())
        .map(|(index, _)| index)
        .expect("应有非首位且没有本地译文的候选");
    for _ in 0..index {
        press(&mut router, function_key(0x28));
    }
    toggle(&mut router);
    let old = translations(&predictor).pop().unwrap();

    let (_, _, chinese) = press(&mut router, function_key(0x28));
    assert!(!chinese.candidates.items.iter().any(|candidate| {
        candidate.text == old.text
            && candidate
                .translation
                .as_ref()
                .is_some_and(|translation| translation.senses()[0].text == "Stale translation.")
    }));
    queue_translation(&predictor, &old, Some("Stale translation."));
    let (after, _) = poll(&mut router);
    assert!(!after.candidates.items.iter().any(|candidate| {
        candidate
            .translation
            .as_ref()
            .is_some_and(|translation| translation.senses()[0].text == "Stale translation.")
    }));
    assert_eq!(translations(&predictor).len(), 1);
}

#[test]
fn clear_and_focus_loss_discard_late_translation_results() {
    for clear in [true, false] {
        let (mut router, predictor) = controlled_router();
        type_letters(&mut router, "xiangkaifa");
        toggle(&mut router);
        let old = translations(&predictor).pop().unwrap();
        if clear {
            press(&mut router, function_key(0x1B));
        } else {
            router.handle(ClientMessage::Commit { session: SESSION });
        }

        queue_translation(&predictor, &old, Some("Stale translation."));
        let (after, commit) = poll(&mut router);
        assert_eq!(commit, None);
        assert!(after.is_empty());
        assert_eq!(translations(&predictor).len(), 1);
    }
}

#[test]
fn space_waits_without_losing_pinyin_then_commits_the_returned_translation() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    let request = translations(&predictor).pop().unwrap();

    let (_, commit, pending) = press(&mut router, punct(' '));
    assert_eq!(commit, None);
    assert_eq!(preedit(&pending), "xiang'kai'fa");
    queue_translation(&predictor, &request, Some("I want to develop it."));
    let (translated, _) = poll(&mut router);
    assert!(translated.candidates.items.iter().any(|candidate| {
        candidate.kind == CandidateKind::Sentence
            && candidate.text == "I want to develop it."
            && candidate
                .translation
                .as_ref()
                .is_some_and(|translation| translation.senses()[0].text == "想开发")
    }));

    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("I want to develop it."));
    assert!(after.is_empty());
}

#[test]
fn failed_translation_ends_pending_state_and_ctrl_retries() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    toggle(&mut router);
    let first = translations(&predictor).pop().unwrap();
    queue_translation(&predictor, &first, None);
    let (failed, _) = poll(&mut router);
    assert!(
        failed
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains("失败"))
    );

    toggle(&mut router);
    let requests = translations(&predictor);
    assert_eq!(requests.len(), 2);
    assert!(requests[1].sequence > first.sequence);
    assert_eq!(requests[1].text, first.text);
}
