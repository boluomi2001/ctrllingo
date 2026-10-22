# 0.1.0 已知问题与正式版门槛

## 依赖安全审查未通过

2026-10-23，GitHub RustSec 审查报告 `zip 8.6.0` 的三条安全公告：

- [RUSTSEC-2026-0369 / GHSA-4684-h2q9-2vcp](https://github.com/zip-rs/zip2/security/advisories/GHSA-4684-h2q9-2vcp)：解包时通过符号链接修改 Unix 权限。
- [RUSTSEC-2026-0367 / GHSA-vv5m-jxxh-4627](https://github.com/zip-rs/zip2/security/advisories/GHSA-vv5m-jxxh-4627)：解包保留 Unix 特权位。
- [RUSTSEC-2026-0368 / GHSA-4v9h-p7m9-rh26](https://github.com/zip-rs/zip2/security/advisories/GHSA-4v9h-p7m9-rh26)：解包建立目录之外的符号链接。

锁定的依赖链是 `qingjian-neural → candle-core 0.11.0 → zip 8.6.0`。Candle 在 NumPy/Pickle 辅助模块中读取 ZIP；本项目的模型入口是 `.qjm` / safetensors，源码检索未发现对 `ZipArchive::extract` 或 `ZipStreamReader::extract` 的调用。这是静态可达性检查，不是完整安全证明，也不表示所有平台不受影响。

本次核实的 Candle 最新稳定发行仍为 0.11.0，其 ZIP 依赖要求不直接接受 9.x。ZIP 上游公告列出 9.0.1 为修复版本；后续需要验证兼容升级或有出处的回补，再运行模型、输入和跨架构回归。本版没有通过忽略公告或关闭审查来制造绿灯。

审查还报告 `ttf-parser 0.25.1` 停止维护的提示；这与已确认的可利用漏洞不是同一个结论，需要后续评估其替代路径。

## 原生验收尚未完成

- 原生安装、重复安装、升级、卸载与重启后的行为。
- 记事本、浏览器和 32 位应用中的真实 TSF 按键、候选与上屏。
- 真实服务商请求、真实余额、价格表与实际账单核对。
- 多屏、不同 DPI、系统搜索和商店应用兼容性。

程序尚无受信任签名，`uiAccess=false`。系统安全策略可能阻止运行；部分系统界面可能遮挡候选窗口。不要通过关闭安全保护解决。

这些条件未完成前，0.1.0 保持 prerelease，不作为稳定正式版推荐。
