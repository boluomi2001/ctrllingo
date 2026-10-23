//! 回归覆盖 candle 的 NPZ 存储 API 与随包 QJM 模型推理。

use std::time::{SystemTime, UNIX_EPOCH};

use candle_core::{DType, Device, Tensor, npy};
use qingjian_neural::CharScorer;

#[test]
fn candle_npz_round_trip() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "qingjian-neural-npz-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("tensors.npz");
    let matrix = Tensor::new(&[[1f32, 2.], [3., 4.]], &Device::Cpu).unwrap();
    let vector = Tensor::new(&[7i64, 11, 13], &Device::Cpu).unwrap();
    Tensor::write_npz(&[("矩阵", &matrix), ("vector", &vector)], &path).unwrap();

    let tensors = Tensor::read_npz(&path).unwrap();
    assert_eq!(tensors.len(), 2);
    assert_eq!(tensors[0].0, "矩阵");
    assert_eq!(tensors[0].1.dims(), &[2, 2]);
    assert_eq!(tensors[0].1.dtype(), DType::F32);
    assert_eq!(tensors[0].1.to_vec2::<f32>().unwrap(), [[1., 2.], [3., 4.]]);
    assert_eq!(tensors[1].0, "vector");
    assert_eq!(tensors[1].1.dims(), &[3]);
    assert_eq!(tensors[1].1.dtype(), DType::I64);
    assert_eq!(tensors[1].1.to_vec1::<i64>().unwrap(), [7, 11, 13]);

    let named = Tensor::read_npz_by_name(&path, &["vector"]).unwrap();
    assert_eq!(named[0].dims(), &[3]);
    assert_eq!(named[0].dtype(), DType::I64);
    assert_eq!(named[0].to_vec1::<i64>().unwrap(), [7, 11, 13]);
    assert!(Tensor::read_npz_by_name(&path, &["missing"]).is_err());
    let lazy = npy::NpzTensors::new(&path).unwrap();
    let loaded = lazy.get("矩阵").unwrap().unwrap();
    assert_eq!(loaded.dims(), &[2, 2]);
    assert_eq!(loaded.dtype(), DType::F32);
    assert_eq!(loaded.to_vec2::<f32>().unwrap(), [[1., 2.], [3., 4.]]);
    let loaded_vector = lazy.get("vector").unwrap().unwrap();
    assert_eq!(loaded_vector.dims(), &[3]);
    assert_eq!(loaded_vector.dtype(), DType::I64);
    assert_eq!(loaded_vector.to_vec1::<i64>().unwrap(), [7, 11, 13]);
    assert!(lazy.get("missing").unwrap().is_none());

    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
#[ignore = "需要已校验的发行模型"]
fn bundled_qjm_models_preserve_scores() {
    let data = std::env::var_os("CTRLLINGO_MODEL_DATA")
        .expect("CTRLLINGO_MODEL_DATA must point to the data root containing models/");
    let models = std::path::PathBuf::from(data).join("models");
    let zhiwei =
        CharScorer::load(&models.join("hanzhang-zhiwei/hanzhang-zhiwei-small.qjm")).unwrap();
    let candidates = ["上海", "伤害", "吃饭"];
    let scores = zhiwei.score("我今天想去", &candidates).unwrap();
    assert_eq!(scores.len(), candidates.len());
    assert!(scores.iter().all(|score| score.is_finite() && *score < 0.));
    assert!(scores[0] > scores[1], "{scores:?}");
    let alone = zhiwei.score("我今天想去", &["伤害"]).unwrap();
    assert!((alone[0] - scores[1]).abs() < 1e-3);
    zhiwei.score("换一段上下文", &["上海"]).unwrap();
    let again = zhiwei.score("我今天想去", &["上海"]).unwrap();
    assert!((again[0] - scores[0]).abs() < 1e-3);

    let tongbian =
        CharScorer::load(&models.join("hanzhang-tongbian/hanzhang-tongbian-small.qjm")).unwrap();
    let p2c = tongbian.score_p2c("jiekou", &["接口", "借口"]).unwrap();
    assert_eq!(p2c.len(), 2);
    assert!(p2c.iter().all(|score| score.is_finite() && *score < 0.));
    for (text, score) in ["接口", "借口"].iter().zip(&p2c) {
        let single = tongbian.score_p2c("jiekou", &[*text]).unwrap();
        assert!((single[0] - score).abs() < 1e-3);
    }
    let empty = tongbian.score_p2c("jiekou", &[]).unwrap();
    assert!(empty.is_empty());
    tongbian
        .score_p2c("womenmingtiankaihui", &["我们明天开会"])
        .unwrap();
    let restored = tongbian.score_p2c("jiekou", &["接口", "借口"]).unwrap();
    for (before, after) in p2c.iter().zip(restored) {
        assert!((before - after).abs() < 1e-3);
    }
}
