//! 首选保护的优势阈值和适用路径测试。

use std::time::{Duration, Instant};

use crate::sentence::{Conversion, SentenceScorer, SentenceWord};

use super::{engine, texts};

struct FixedScores {
    favored: &'static str,
    score: f64,
    offset: f64,
}

impl SentenceScorer for FixedScores {
    fn score(&self, _context: &str, _keys: &str, texts: &[&str]) -> Vec<f64> {
        texts
            .iter()
            .map(|text| {
                self.offset
                    + if *text == self.favored {
                        self.score
                    } else {
                        0.0
                    }
            })
            .collect()
    }
}

fn path(text: &str, syllables: &[&str], word_parts: &[(&str, &[&str])]) -> Conversion {
    Conversion {
        text: text.to_owned(),
        syllables: syllables.iter().map(|s| (*s).to_owned()).collect(),
        words: word_parts
            .iter()
            .map(|(text, syllables)| SentenceWord {
                text: (*text).to_owned(),
                syllables: syllables.iter().map(|s| (*s).to_owned()).collect(),
                placeholder: false,
            })
            .collect(),
        score: -10.0,
        static_score: -10.0,
        penalty: 0.0,
    }
}

fn four_syllable_paths() -> Vec<Conversion> {
    vec![
        path(
            "我想看书",
            &["wo", "xiang", "kan", "shu"],
            &[
                ("我", &["wo"]),
                ("想", &["xiang"]),
                ("看书", &["kan", "shu"]),
            ],
        ),
        path(
            "我想砍树",
            &["wo", "xiang", "kan", "shu"],
            &[
                ("我", &["wo"]),
                ("想", &["xiang"]),
                ("砍树", &["kan", "shu"]),
            ],
        ),
    ]
}

fn three_syllable_paths() -> Vec<Conversion> {
    vec![
        path(
            "同因此",
            &["tong", "yin", "ci"],
            &[("同", &["tong"]), ("因此", &["yin", "ci"])],
        ),
        path(
            "同音词",
            &["tong", "yin", "ci"],
            &[("同音", &["tong", "yin"]), ("词", &["ci"])],
        ),
    ]
}

fn rescored_order(
    mut paths: Vec<Conversion>,
    favored: &'static str,
    score: f64,
    offset: f64,
    async_mode: bool,
) -> Vec<String> {
    let scorer = Box::new(FixedScores {
        favored,
        score,
        offset,
    });
    let keys = paths[0].syllables.concat();
    if async_mode {
        let mut engine = engine().with_async_sentence_scorer(scorer, Some(0.5), None, None);
        engine.rescore_paths(&mut paths, &keys);
        assert!(engine.request_rescoring());
        let started = Instant::now();
        while !engine.poll_rescoring() {
            assert!(started.elapsed() < Duration::from_secs(5), "后台没回结果");
            std::thread::sleep(Duration::from_millis(5));
        }
        engine.rescore_paths(&mut paths, &keys);
    } else {
        engine()
            .with_sentence_scorer(scorer, Some(0.5), None, None)
            .rescore_paths(&mut paths, &keys);
    }
    texts(&paths).into_iter().map(str::to_owned).collect()
}

#[test]
fn long_multiword_sentences_need_one_nat_but_short_sentences_do_not() {
    let long = four_syllable_paths();
    assert_eq!(
        rescored_order(long.clone(), "我想砍树", 1.0, 0.0, false),
        ["我想看书", "我想砍树"]
    );
    assert_eq!(
        rescored_order(long.clone(), "我想砍树", 3.0, 0.0, false),
        ["我想砍树", "我想看书"]
    );
    assert_eq!(
        rescored_order(long, "我想砍树", 3.0, 100.0, false),
        ["我想砍树", "我想看书"]
    );
    assert_eq!(
        rescored_order(three_syllable_paths(), "同音词", 1.0, 0.0, false),
        ["同音词", "同因此"]
    );
}

#[test]
fn async_rescoring_uses_the_same_long_sentence_rule() {
    let long = four_syllable_paths();
    assert_eq!(
        rescored_order(long.clone(), "我想砍树", 1.0, 0.0, true),
        ["我想看书", "我想砍树"]
    );
    assert_eq!(
        rescored_order(long, "我想砍树", 3.0, 0.0, true),
        ["我想砍树", "我想看书"]
    );
}
