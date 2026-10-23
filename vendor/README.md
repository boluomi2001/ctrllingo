# Vendored candle-core

本目录来自 crates.io 官方发布包 `candle-core 0.11.0`：https://crates.io/api/v1/crates/candle-core/0.11.0/download

原始 `.crate` SHA-256：`5ecb245093b0f791b89d3420c3df9c6d49c60ab63ba54db896bf8a3baf486706`。

相对官方包仅修改 `Cargo.toml` 中 `[dependencies.zip]` 的 `version`，从 `8.6.0` 改为 `9.0.5`；`default-features = false` 保持不变。此版本修复 GHSA-4684-h2q9-2vcp、GHSA-vv5m-jxxh-4627 与 GHSA-4v9h-p7m9-rh26。另在 `src/npy.rs` 和 `src/pickle.rs` 为 ZIP 9 的文件名 API 加入错误传播与 `Cow<str>` 兼容处理；推理与张量计算代码未改。

未来升级到包含该依赖修复的上游 candle-core 后，应移除此目录与根 `Cargo.toml` 的 crates.io patch。
