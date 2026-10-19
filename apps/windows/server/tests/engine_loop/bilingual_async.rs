//! 整句译文异步返回时的序号过滤、取消与私密边界。

use std::collections::VecDeque;

use crate::support::*;
use qingjian_core::{
    CandidateKind, Prediction, PredictionKind, PredictionPolicy, PredictionRequest, Predictor,
};

#[derive(Clone)]
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
    let predictor = ControlledPredictor {
        submitted: Arc::new(Mutex::new(Vec::new())),
        replies: Arc::new(Mutex::new(VecDeque::new())),
    };
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

fn last_translation(submitted: &Arc<Mutex<Vec<PredictionRequest>>>) -> PredictionRequest {
    submitted
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find(|request| request.kind == PredictionKind::Translate)
        .expect("当前整句应提交 Translate 请求")
        .clone()
}

fn queue_translation(
    replies: &Arc<Mutex<VecDeque<Prediction>>>,
    request: &PredictionRequest,
    text: &str,
) {
    replies.lock().unwrap().push_back(Prediction {
        sequence: request.sequence,
        words: Vec::new(),
        sentence: Some(text.to_owned()),
    });
}

fn poll(router: &mut Router) -> (Frame, Option<String>) {
    match router.handle(ClientMessage::Poll { session: SESSION }) {
        Some(ServerMessage::Update { frame, commit, .. }) => (frame, commit),
        other => panic!("expected Update, got {other:?}"),
    }
}

fn translated_sentence(frame: &Frame, text: &str) {
    assert!(
        frame.candidates.items.iter().any(|candidate| {
            candidate.kind == CandidateKind::Sentence && candidate.text == text
        })
    );
}

#[test]
fn newer_sentence_translation_wins_when_replies_arrive_out_of_order() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    let old = last_translation(&predictor.submitted);

    press(&mut router, function_key(KeyEvent::SWAP_TRANSLATION));
    type_letters(&mut router, "zhe");
    let current = last_translation(&predictor.submitted);
    assert!(current.sequence > old.sequence);
    assert_ne!(current.text, old.text);

    queue_translation(&predictor.replies, &current, "New complete sentence.");
    let (frame, commit) = poll(&mut router);
    assert_eq!(commit, None);
    translated_sentence(&frame, "New complete sentence.");

    queue_translation(&predictor.replies, &old, "Stale sentence.");
    let (frame, commit) = poll(&mut router);
    assert_eq!(commit, None);
    translated_sentence(&frame, "New complete sentence.");
    assert!(!candidate_texts(&frame).contains(&"Stale sentence."));

    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("New complete sentence."));
    assert!(after.is_empty());
}

#[test]
fn pending_translation_does_not_commit_or_clear_until_reply_arrives() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    let request = last_translation(&predictor.submitted);
    press(&mut router, function_key(KeyEvent::SWAP_TRANSLATION));

    let (_, commit, pending) = press(&mut router, punct(' '));
    assert_eq!(commit, None);
    assert_eq!(preedit(&pending), "xiang'kai'fa");
    assert!(!pending.is_empty());

    queue_translation(&predictor.replies, &request, "Complete sentence.");
    let (translated, commit) = poll(&mut router);
    assert_eq!(commit, None);
    translated_sentence(&translated, "Complete sentence.");
    let (_, commit, after) = press(&mut router, punct(' '));
    assert_eq!(commit.as_deref(), Some("Complete sentence."));
    assert!(after.is_empty());
}

#[test]
fn cancelled_translation_cannot_reopen_after_escape() {
    let (mut router, predictor) = controlled_router();
    type_letters(&mut router, "xiangkaifa");
    let request = last_translation(&predictor.submitted);
    let (_, commit, cleared) = press(&mut router, function_key(0x1B));
    assert_eq!(commit, None);
    assert!(cleared.is_empty());

    queue_translation(&predictor.replies, &request, "Late sentence.");
    let (after, commit) = poll(&mut router);
    assert_eq!(commit, None);
    assert!(after.is_empty());

    let (_, _, chinese) = type_letters(&mut router, "nihao");
    assert_eq!(chinese.candidates.items[0].text, "你好");
}

#[test]
fn deleting_or_committing_the_buffer_resets_pending_translation() {
    for commit_buffer in [false, true] {
        let (mut router, predictor) = controlled_router();
        type_letters(&mut router, "xiangkaifa");
        let request = last_translation(&predictor.submitted);
        press(&mut router, function_key(KeyEvent::SWAP_TRANSLATION));

        if commit_buffer {
            assert_eq!(
                router.handle(ClientMessage::Commit { session: SESSION }),
                Some(ServerMessage::Committed {
                    session: SESSION,
                    text: Some("xiangkaifa".to_owned()),
                })
            );
        } else {
            let mut frame = Frame::default();
            for _ in "xiangkaifa".chars() {
                let (_, _, next) = press(&mut router, function_key(0x08));
                frame = next;
            }
            assert!(frame.is_empty());
        }

        queue_translation(&predictor.replies, &request, "Late sentence.");
        let (after, commit) = poll(&mut router);
        assert_eq!(commit, None);
        assert!(after.is_empty());

        let (_, _, chinese) = type_letters(&mut router, "nihao");
        assert_eq!(chinese.candidates.items[0].text, "你好");
    }
}

#[test]
fn private_composition_submits_no_sentence_translation() {
    let (mut router, predictor) = controlled_router();
    router.handle(ClientMessage::Privacy {
        session: SESSION,
        private: true,
    });
    type_letters(&mut router, "xiangkaifa");

    assert!(router.is_private());
    assert!(
        predictor
            .submitted
            .lock()
            .unwrap()
            .iter()
            .all(|request| request.kind != PredictionKind::Translate)
    );
}
