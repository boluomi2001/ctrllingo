# 参与贡献

感谢你愿意帮助改进 CtrlLingo · 译点。项目是 Qingjian 的独立 Windows 二次开发版本；提交前请先阅读 [`docs/UPSTREAM.md`](docs/UPSTREAM.md)、仓库中的 `LICENSE`，以及改动涉及目录内的数据和组件许可。

## 提交前先沟通

先查看 [现有 Issues](https://github.com/boluomi2001/ctrllingo/issues)，确认没有重复问题。较大的功能、兼容性变化或涉及多个模块的改动，建议先开 issue 讨论目标和范围。

## Pull Request

- 每个 PR 聚焦一个问题，尽量保持改动小且容易审查。
- 描述问题、预期行为、实际改动和验证方式。修复输入行为时，请记录 Windows 版本、目标应用、复现步骤和修复后的结果。
- 本项目当前发布目标是 Windows x64（含供 32 位应用使用的 x86 TSF）。其他平台上游代码不属于本版本的发布验收范围。
- 不要把 API 密钥、个人输入、真实用户日志或其他敏感信息放进提交和 issue。新增数据或素材时，写明来源和许可。
- 不要在贡献中移除 Qingjian 上游版权、作者归属或 GPL-3.0-or-later 声明。

PR 需由维护者审查后合并。提交 PR 不代表改动已通过审核或进入发布版本。

## 本地构建

Windows 安装包构建需要 Rust 1.96、MSVC C++ 工具链和 Inno Setup 6。PowerShell 7 或 Windows PowerShell 5.1 可从仓库根目录运行：

```powershell
$env:CTRLLINGO_ISCC = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
.\apps\windows\installer\build-ctrllingo.ps1
```

脚本按 [`tools/release/data.lock`](tools/release/data.lock) 获取并校验产品数据，默认把安装包、源代码快照和 `SHA256SUMS.txt` 写入 `target\ctrllingo-release`。如需运行 Windows 相关 crate 的测试：

```powershell
cargo test --locked -p qingjian-windows-server -p qingjian-windows-tsf -p qingjian-settings
cargo test --locked -p qingjian-core
```

这些是本地验证命令，不表示发布验收已经完成。若本地环境无法完成真实 TSF 上屏和安装器升级/卸载验证，请在 PR 中明确写出已完成和未完成的检查；编译或自动化测试不能代替这些验证。

当前发行版本为 0.1.1 Windows 依赖安全维护版（未签名）。原生安装、升级、卸载和真实 TSF 上屏等验收状态见[发布验收记录](docs/RELEASE-VALIDATION.md)；提交 PR 不代表改动已审核或进入发布版本。

## 许可

除另有明确许可说明的文件或组件外，贡献按 GPL-3.0-or-later 提供。提交者应有权提交所贡献的代码、文档和素材，并在 PR 中注明第三方来源。
