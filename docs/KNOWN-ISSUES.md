# CtrlLingo 已知问题与验证范围

## 0.1.1 依赖审查结果

0.1.0 的锁文件包含 `qingjian-neural → candle-core 0.11.0 → zip 8.6.0`，RustSec 报告了三条 ZIP 公告：

- [RUSTSEC-2026-0369 / GHSA-4684-h2q9-2vcp](https://github.com/zip-rs/zip2/security/advisories/GHSA-4684-h2q9-2vcp)：解包时通过符号链接修改 Unix 权限。
- [RUSTSEC-2026-0367 / GHSA-vv5m-jxxh-4627](https://github.com/zip-rs/zip2/security/advisories/GHSA-vv5m-jxxh-4627)：解包保留 Unix 特权位。
- [RUSTSEC-2026-0368 / GHSA-4v9h-p7m9-rh26](https://github.com/zip-rs/zip2/security/advisories/GHSA-4v9h-p7m9-rh26)：解包建立目录之外的符号链接。

0.1.1 将 ZIP 更新至 9.0.5，并为 `candle-core 0.11.0` 的 ZIP 文件名 API 增加兼容补丁。vendored 官方包版本、原始 `.crate` SHA-256 与补丁来源见 [`vendor/README.md`](../vendor/README.md)。使用官方 `cargo-audit 0.22.2` 和同一 RustSec 数据库提交 `f7b2f6bc663452200bbe639ebb75e0c762b3422b` 对比扫描，旧锁文件报告 3 条漏洞，新锁文件报告 0 条，退出码为 0。该结果仅描述该锁文件与数据库快照，不代表软件不存在漏洞。

现有 `.cargo/audit.toml` 对 `paste` 的忽略项 `RUSTSEC-2024-0436` 保持不变；本次没有新增忽略项。扫描同时保留 `ttf-parser 0.25.1` 停止维护提醒 [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192)。停止维护提醒与上述 ZIP 漏洞不是同一类结论，后续替代评估仍待处理。

## 0.1.1 未覆盖范围

本次 890 项本地回归、额外的两款发行 QJM 模型验证、格式和严格 Clippy 检查、x64/x86 构建与 DLL 加载均已通过。0.1.1 原生安装、升级、卸载、物理键盘实际上屏、不同应用和 DPI、真实 API 账户及账单尚未单独验证。详见[发布验收记录](RELEASE-VALIDATION.md)。

程序未签名且 `uiAccess=false`。Windows 安全策略可能拦截程序；系统搜索等界面中的候选窗可能被遮挡。签名状态与软件是否商业发行无关；请勿关闭系统安全保护来绕过提示。

## 0.1.0 历史记录

用户于 2026-10-23 反馈“我测试完了，所有功能都是正常的”。这是 0.1.0 的历史用户反馈，不是 0.1.1 安装升级或所有应用兼容性的测试证据。0.1.0 时的未单独记录范围包括 32 位应用、多屏和不同 DPI、系统搜索与商店应用、跨版本升级、重复安装、卸载/重启状态及真实服务账单核对。记录缺口不等于测试失败，也不证明这些场景已通过。
