//! 用仓库内的含章模型对照整句静态路径打分。
//!
//! Run from the repository root with `cargo run -p qingjian-cli --example rescore_probe`.

use std::path::Path;

use qingjian_core::{
    parser,
    sentence::{self, Personal, SentenceScorer, SpanCache},
};
use qingjian_dictionary::Dictionary;
use qingjian_lm::BigramModel;
use qingjian_neural::{CharScorer, P2cScorer};

const CASES: &[(&str, &str)] = &[
    (
        "woxiangdawanpingpangqiuhouqukandianying",
        "我想打完乒乓球后去看电影",
    ),
    ("woxiangdawanlanqiuhouquchifan", "我想打完篮球后去吃饭"),
    ("woxiangdawanyouxihouqushuijiao", "我想打完游戏后去睡觉"),
];

const PAIRS: &[(&str, &str, &str)] = &[
    (
        "woxiangdawanpingpangqiuhouqukandianying",
        "我想打完乒乓球后去看电影",
        "我想打玩乒乓球后去看电影",
    ),
    (
        "woxiangdawanlanqiuhouquchifan",
        "我想打完篮球后去吃饭",
        "我想打玩篮球后去吃饭",
    ),
    (
        "woxiangdawanyouxihouqushuijiao",
        "我想打完游戏后去睡觉",
        "我想打玩游戏后去睡觉",
    ),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = Dictionary::from_path("data/generated/dict.qj")?;
    let model = BigramModel::from_path(Path::new("data/generated/lm.qj"))?;
    let models = [
        ("hanzhang-tongbian", "data/models/hanzhang-tongbian"),
        ("hanzhang-zhiwei", "data/models/hanzhang-zhiwei"),
    ];

    for (model_name, model_path) in models {
        let scorer = load_scorer(Path::new(model_path))?;
        println!("\n=== {model_name} ===");
        for (keys, expected) in CASES {
            let segmentation = parser::segment(keys)?[0].clone();
            let positions: Vec<Vec<_>> = segmentation
                .patterns()
                .into_iter()
                .map(|pattern| vec![pattern])
                .collect();
            let paths = sentence::convert_paths(
                &[&dictionary],
                &positions,
                false,
                6,
                &model,
                Personal::NONE,
                |_| 0,
                |_, _| 0.0,
                &mut SpanCache::default(),
            );
            let texts: Vec<&str> = paths.iter().map(|path| path.text.as_str()).collect();
            let neural = scorer.score("", keys, &texts);
            println!("\nkeys={keys} segmentation={segmentation} expected={expected}");
            println!("correct_in_top6={}", texts.contains(expected));
            if neural.len() != paths.len() {
                println!("neural_scores_unavailable");
            } else {
                let mut ranked: Vec<_> = paths
                    .iter()
                    .zip(neural)
                    .map(|(path, score)| {
                        let mixed = path.score + 0.5 * (score - path.static_score);
                        (path, score, mixed)
                    })
                    .collect();
                ranked.sort_by(|a, b| b.2.total_cmp(&a.2));
                for (rank, (path, score, mixed)) in ranked.iter().enumerate() {
                    let words = path
                        .words
                        .iter()
                        .map(|word| word.text.as_str())
                        .collect::<Vec<_>>()
                        .join("/");
                    println!(
                        "{:>2}. text={} words={} score={:.4} static_score={:.4} neural={:.4} mixed={:.4} gap={:.4}",
                        rank + 1,
                        path.text,
                        words,
                        path.score,
                        path.static_score,
                        score,
                        mixed,
                        ranked.first().map_or(0.0, |top| top.2 - mixed),
                    );
                }
            }
        }

        println!("\nfull-text pair scores (positive delta favors correct)");
        for (keys, correct, wrong) in PAIRS {
            let scores = scorer.score("", keys, &[correct, wrong]);
            if scores.len() == 2 {
                println!(
                    "keys={keys} correct={correct} {:.4} wrong={wrong} {:.4} delta={:.4}",
                    scores[0],
                    scores[1],
                    scores[0] - scores[1],
                );
            } else {
                println!("keys={keys} pair_scores_unavailable");
            }
        }
    }
    Ok(())
}

fn load_scorer(path: &Path) -> Result<Box<dyn SentenceScorer>, Box<dyn std::error::Error>> {
    let scorer = CharScorer::load(path)?;
    if scorer.vocab().sep().is_some() {
        Ok(Box::new(P2cScorer(scorer)))
    } else {
        Ok(Box::new(scorer))
    }
}
