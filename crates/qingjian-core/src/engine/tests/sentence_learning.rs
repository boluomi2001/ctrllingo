//! 整句备选的个人 n-gram 学习与撤销。

use super::*;

type SharedLearning = Arc<Mutex<(Vec<String>, sentence::UserNgram)>>;

fn sentence_engine() -> (Engine, SharedLearning) {
    let dictionary = Dictionary::parse(
        "我\two\t100000\n想\txiang\t100000\n看书\tkan shu\t100000\n砍树\tkan shu\t1000\n",
    )
    .unwrap();
    let shared = Arc::new(Mutex::new((Vec::new(), sentence::UserNgram::default())));
    let learner = WordLearner {
        shared: Arc::clone(&shared),
        ..WordLearner::default()
    };
    (
        Engine::new(dictionary).with_learner(Box::new(learner)),
        shared,
    )
}

fn sentences(engine: &mut Engine) -> Vec<Candidate> {
    engine.set_input("woxiangkanshu");
    engine
        .query()
        .unwrap()
        .candidates
        .items
        .into_iter()
        .filter(|candidate| candidate.kind == CandidateKind::Sentence)
        .collect()
}

#[test]
fn alternate_sentence_is_learned_twice_and_retraction_removes_it() {
    let (mut engine, shared) = sentence_engine();
    let candidates = sentences(&mut engine);
    assert_eq!(
        candidates
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>(),
        ["我想看书", "我想砍树"]
    );
    let static_first = candidates[0].clone();
    let alternate = candidates[1].clone();
    engine.commit(&alternate);
    {
        let ngram = &shared.lock().unwrap().1;
        assert_eq!(ngram.pair(None, "我"), EXPLICIT_TRANSITION_WEIGHT);
        assert_eq!(ngram.pair(Some("我"), "想"), EXPLICIT_TRANSITION_WEIGHT);
        assert_eq!(ngram.pair(Some("想"), "砍树"), EXPLICIT_TRANSITION_WEIGHT);
        assert_eq!(ngram.pair(Some("想"), "看书"), 0);
    }

    for _ in 0..alternate.text.chars().count() {
        engine.note_backspace();
    }
    let candidates = sentences(&mut engine);
    let rank = candidates
        .iter()
        .position(|candidate| candidate.text == static_first.text)
        .unwrap();
    let replacement = candidates[rank].clone();
    engine.commit(&replacement);
    let ngram = &shared.lock().unwrap().1;
    assert_eq!(ngram.pair(Some("想"), "砍树"), 0);
    assert_eq!(
        ngram.pair(Some("想"), "看书"),
        if rank == 0 {
            1
        } else {
            EXPLICIT_TRANSITION_WEIGHT
        }
    );

    let (mut fresh, fresh_shared) = sentence_engine();
    let default = sentences(&mut fresh).remove(0);
    fresh.commit(&default);
    let ngram = &fresh_shared.lock().unwrap().1;
    assert_eq!(ngram.pair(None, "我"), 1);
    assert_eq!(ngram.pair(Some("我"), "想"), 1);
    assert_eq!(ngram.pair(Some("想"), "看书"), 1);
}

#[test]
fn an_unmatched_sentence_text_does_not_learn_a_guessed_path() {
    let (mut engine, shared) = sentence_engine();
    engine.set_input("woxiangkanshu");
    let forged = Candidate {
        text: "我想伪造的句子".into(),
        kind: CandidateKind::Sentence,
        syllables: ["wo", "xiang", "kan", "shu"].map(str::to_owned).to_vec(),
        reading: None,
        translation: None,
        aux_code: None,
    };
    engine.commit(&forged);
    let ngram = &shared.lock().unwrap().1;
    assert_eq!(ngram.pair(None, "我"), 0);
    assert_eq!(ngram.pair(Some("我"), "想"), 0);
}
