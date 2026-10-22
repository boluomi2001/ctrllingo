<p align="center">
  <img src="assets/brand/ctrllingo-mark.png" alt="CtrlLingo 译点标志" width="160">
</p>

<h1 align="center">CtrlLingo · 译点</h1>

<p align="center"><strong>边输入，边翻译；让每个词都有落点。</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/平台-Windows%20x64-blue" alt="Windows x64">
  <img src="https://img.shields.io/badge/许可-GPL--3.0--or--later-blue" alt="GPL-3.0-or-later">
  <img src="https://img.shields.io/badge/版本-0.1.0-orange" alt="Version 0.1.0">
</p>

CtrlLingo · 译点是一款 Windows 拼音输入法，基于 Qingjian（青简）上游项目独立二次开发。当前准备发布 **0.1.0 预发布版**，发布目标为 Windows x64 主程序，并附带供 32 位应用使用的 x86 TSF 组件。它尚未完成原生安装、升级、卸载和真实 TSF 上屏验收，因此不称为稳定版。

## 功能

- 拼音输入、候选查询、学习和本地整句模型可在本机运行；拼音查询不需要联网。
- 按 Ctrl 可请求当前拼音组合中**高亮中文候选**的译文。它不会读取或翻译其他应用里已选中的文字。目标语言可选英语、日语、德语、西班牙语和越南语。
- 长句重排采用保守策略，尽量不打乱整句选择；整句学习会根据你的选择调整后续候选顺序。
- 云联想默认关闭，且不会自动请求。按需翻译是独立的显式请求功能；开启它仍会把当前高亮候选发给你配置的兼容 API。云请求需要你自备地址和 API 密钥，服务商可能收费。

## 三步开始输入

1. 安装后按 **Win + Space**，在 Windows 输入法列表中选择“译点 CtrlLingo”。
2. 在应用中输入一整句拼音，检查并移动到你想要的中文候选。
3. 单独松开 **Ctrl** 请求该候选的译文；译文准备好后按 **空格**上屏。按 Ctrl 可切回中文候选；改字或改选后，再按 Ctrl 请求新候选的译文。

若没有配置兼容 API 地址和密钥，云端译文不会返回。API 密钥可在设置程序的“云服务”页填写；云联想关闭时，仍可按 Ctrl 主动请求整句翻译。

## 下载、安装与卸载

从 [GitHub Releases](https://github.com/boluomi2001/ctrllingo/releases) 下载 `CtrlLingo-0.1.0-windows-x64-setup.exe`。0.1.0 将以预发布版本提供，直到原生安装、升级、卸载和真实 TSF 上屏验收完成。当前没有签名；请核对发布页提供的 `SHA256SUMS.txt`，不要关闭 Windows 安全保护来绕过提示。

运行安装包并按提示完成安装。安装程序会注册 x64 与 x86 TSF；首次切换输入法时，如果应用已经打开，请重新打开应用，或注销后重新登录。

卸载可从开始菜单的“卸载 CtrlLingo”入口，或 Windows“设置 → 应用”中启动。卸载会移除程序和 TSF 注册，但保留用户配置、密钥、学习记录、统计和运行日志；如需删除个人数据，请在卸载后另行删除 `%APPDATA%\CtrlLingo` 与 `%LOCALAPPDATA%\CtrlLingo`。

安装目录为 `%ProgramFiles%\CtrlLingo`，后台进程和设置程序分别命名为 `ctrllingo-server.exe` 与 `ctrllingo-settings.exe`；Windows 输入法列表和托盘显示“译点 CtrlLingo”，设置窗口名为“译点设置”。配置、密钥和学习数据位于 `%APPDATA%\CtrlLingo`；运行日志位于 `%LOCALAPPDATA%\CtrlLingo\logs`；机器级 TSF 图标位于 `%ProgramData%\CtrlLingo`。

## 费用估算与账户余额

费用页面按本机记录的请求和价格表估算用量，不是服务商账单。价格变化、缺少服务商用量信息、缓存计价差异或自定义兼容服务都可能导致估算不完整。账户余额是服务商账户的余额，可能包含其他应用的消费和充值，不等于 CtrlLingo 的费用。余额查询只支持代码已实现的 DeepSeek 官方接口。

## 隐私与更新

按需翻译会将当前高亮的中文候选发送到你填写的服务商；云联想开启后会发送光标附近文本。两者使用同一兼容 API 配置，但触发条件不同。API 密钥保存在本机配置文件中。输入日志默认关闭。详细范围见[隐私说明](docs/PRIVACY.md)。

Windows 版本不会向 Qingjian 上游检查更新。请在“关于”页手动打开本项目的 [GitHub Releases](https://github.com/boluomi2001/ctrllingo/releases) 查看新版本。

## 构建与测试

需要 Rust 1.96、MSVC C++ 工具链和 Inno Setup 6。PowerShell 7 或 Windows PowerShell 5.1 可从仓库根目录运行：

```powershell
$env:CTRLLINGO_ISCC = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
.\apps\windows\installer\build-ctrllingo.ps1
```

脚本按 [`tools/release/data.lock`](tools/release/data.lock) 获取并校验产品数据，默认把安装包、源代码快照和 `SHA256SUMS.txt` 写入 `target\ctrllingo-release`。安装包名为 `CtrlLingo-0.1.0-windows-x64-setup.exe`，源码快照名为 `CtrlLingo-0.1.0-source.zip`。可运行以下命令执行 Windows 相关 crate 的测试；这些是验证步骤，不表示本次发布验收已完成：

```powershell
cargo test --locked -p qingjian-windows-server -p qingjian-windows-tsf -p qingjian-settings
cargo test --locked -p qingjian-core
```

## 项目来源与许可

CtrlLingo 是由 `boluomi2001` 维护的独立 Windows 二次开发版本，基于 [Qingjian 上游](https://github.com/qingjian-team/qingjian)提交 [`c97ff2b61933d6fcd845ee4f4f470faf3ea83bf6`](https://github.com/qingjian-team/qingjian/commit/c97ff2b61933d6fcd845ee4f4f470faf3ea83bf6)，并包含当前工作树第四版优化。译点不是青简官方产品，也不声称原创全部输入引擎。上游说明和原始 README 副本见 [`docs/UPSTREAM.md`](docs/UPSTREAM.md) 与 [`docs/upstream/README.zh-CN.md`](docs/upstream/README.zh-CN.md)。

代码采用 [GPL-3.0-or-later](LICENSE)；随包数据和部分组件可能有各自的来源与许可。macOS、Linux 等上游继承平台没有本版本的译点发布验收。

## 反馈与参与

- [报告缺陷或提出建议](https://github.com/boluomi2001/ctrllingo/issues/new/choose)
- [安全政策](SECURITY.md)（请私下报告安全问题）
- [支持说明](SUPPORT.md) · [贡献指南](CONTRIBUTING.md)
