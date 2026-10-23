# 0.1.0 已知问题与正式版门槛

## 依赖安全审查未通过

2026-10-23，GitHub RustSec 审查报告 `zip 8.6.0` 的三条安全公告：

- [RUSTSEC-2026-0369 / GHSA-4684-h2q9-2vcp](https://github.com/zip-rs/zip2/security/advisories/GHSA-4684-h2q9-2vcp)：解包时通过符号链接修改 Unix 权限。
- [RUSTSEC-2026-0367 / GHSA-vv5m-jxxh-4627](https://github.com/zip-rs/zip2/security/advisories/GHSA-vv5m-jxxh-4627)：解包保留 Unix 特权位。
- [RUSTSEC-2026-0368 / GHSA-4v9h-p7m9-rh26](https://github.com/zip-rs/zip2/security/advisories/GHSA-4v9h-p7m9-rh26)：解包建立目录之外的符号链接。

锁定的依赖链是 `qingjian-neural → candle-core 0.11.0 → zip 8.6.0`。Candle 在 NumPy/Pickle 辅助模块中读取 ZIP；本项目的模型入口是 `.qjm` / safetensors，源码检索未发现对 `ZipArchive::extract` 或 `ZipStreamReader::extract` 的调用。这是静态可达性检查，不是完整安全证明，也不表示所有平台不受影响。

本次核实的 Candle 最新稳定发行仍为 0.11.0，其 ZIP 依赖要求不直接接受 9.x。ZIP 上游公告列出 9.0.1 为修复版本；后续需要验证兼容升级或有出处的回补，再运行模型、输入和跨架构回归。本版没有通过忽略公告或关闭审查来制造绿灯。

审查还报告 `ttf-parser 0.25.1` 停止维护的提示；这与已确认的可利用漏洞不是同一个结论，需要后续评估其替代路径。

## 用户反馈与未单独记录的验收项

用户于 2026-10-23 反馈“我测试完了，所有功能都是正常的”，功能验收记为通过（用户反馈）。具体环境和逐项结果未单独记录，以下覆盖范围尚无单独记录：

- 32 位应用、多屏与不同 DPI、系统搜索和商店应用兼容性。
- 跨版本升级、重复安装与卸载/重启后的状态。
- 真实服务请求、真实余额及账单核对。

这些记录缺口不等于测试失败，也不能用于宣称所有环境均兼容。依赖安全审查仍未通过；详细记录见[发布验收](RELEASE-VALIDATION.md)。

## 其他发布限制

程序尚无受信任签名，`uiAccess=false`。系统安全策略可能阻止运行；部分系统界面可能遮挡候选窗口。不要通过关闭安全保护解决。

依赖安全审查未通过，且若干兼容性和生命周期检查未单独记录；处理这些事项前，0.1.0 保持 prerelease，不作为稳定正式版推荐。
