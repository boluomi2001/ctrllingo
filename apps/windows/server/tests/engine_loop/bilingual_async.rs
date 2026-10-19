//! Ctrl 按需翻译的请求次数、当前候选绑定、取消与失败重试。

use std::collections::VecDeque;

use crate::support::*;
use qingjian_core::{
    CandidateKind, Prediction, PredictionKind, PredictionPolicy, PredictionRequest, Predictor,
};

#[derive(Clone, Default)]
struct ControlledPredictor {
    submitted: Arc<Mutex<Vec<PredictionRequest>>>,

    replies: Arc<Mutex<VecDeque<Prediction>>>,
}

impl Predictor for ControlledPredictor {
    fn policy(&self) -> PredictionPolicy {
        PredictionPolicy {
            slots: 0,
            max_items: 0,
            sentence: false,
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
    let predictor = ControlledPredictor::default();
    let mut router = router_with(RouterConfig {
        cloud_enabled: false,
        cloud_slots: 0,
        sentence_translation: true,
        ..RouterConfig::default()
    });
    router
        .engine_mut()
        .set_predictor(Box::new(predictor.clone()));
    (router, predictor)
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
