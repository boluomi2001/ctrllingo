# Ctrl 按需翻译与费用验证记录

## 回滚与隔离

修改前已将上一版可用源码提交为 `2c0ed70221b90fb7f2866fe96564ed4b5258acae`，标签为 `backup/bilingual-live-before-ondemand-20261019`；新改动位于 `feature/ctrl-translation-cost`。完整历史另存于 `target/backups/bilingual-before-ctrl-cost-20261019.bundle`，已通过 `git bundle verify`。

上一版压缩包保留在 `target/bilingual-test/QingjianBilingualTest-windows-x64.zip`，本轮新包使用另一个输出目录。原版输入法的程序、注册、配置和进程未改动。当前“青简双语测试”的安装标识继续使用原测试版标识，新测试版升级需要关闭使用旧 DLL 的应用并按安装说明注销登录。

回看备份源码可用 `git switch --detach backup/bilingual-live-before-ondemand-20261019`；Git 会阻止覆盖有冲突的未提交修改。需要继续开发时切回 `feature/ctrl-translation-cost`。软件回退可重新安装保留的旧测试包；源码回退不会自动替换电脑上的程序。

## 本轮行为

- 普通中文输入不自动请求整句翻译；Ctrl 绑定当前高亮候选，优先使用已有本地英文和精确缓存。
- 等待时空格不误交中文；再次 Ctrl 可撤销等待意图。编辑或改选回中文，旧回复作废，不自动重发。
- Ctrl 按需功能开启期间，旧配置中的自动云联想、释义兜底与生词本云翻译受到抑制；配置热加载也重新应用策略。
- 实际 HTTP 尝试分别记账，响应缺 usage、模型未知、进行中或强制退出未完成不视为免费；缓存属于输入总量的一部分。
- 官方账户余额在后台按需查询，所有应用共用 Server 的 60 秒缓存；余额失败保留旧数和更新时间。输入法估算与账户余额分别标注。
- 只为官方 Flash 及当前支持的别名套用版本化人民币价格。缓存细分或法定节假日判断不全时显示区间，未知费用不伪造精确值。

## 当前验证状态

已通过的检查：六个 crate 的 release 测试共 730 项（`.tools/ctrl-cost-final-tests.log`）；六包 Clippy `--all-targets -- -D warnings`（`.tools/ctrl-cost-clippy.log`）；`cargo fmt` 被 Windows 应用控制拦截后，对 1069 个 Rust 文件完成 direct rustfmt 检查；x64 Server、Settings 和 TSF release 构建（`.tools/ctrl-cost-build64.log`）。构建期间 Windows 应用控制保持开启，使用独立工具目录、Rust GNU 1.96.0，以及 `RUSTFLAGS=-C link-self-contained=no` 和 `CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_OPT_LEVEL=1`。

32 位 TSF release 构建也已通过（`.tools/ctrl-cost-build32.log`）；64 位和 32 位 DLL 均已实际加载并核对四个 COM 导出，未调用注册函数。

新 Server 已通过真实命名管道和本机 HTTP 模拟服务验证（`.tools/ctrl-cost-smoke.log`）：普通词语与整句输入暂停后为零请求，Ctrl 后一次请求，等待时空格保留拼音，编辑后的晚回复被忽略，同句再次 Ctrl 命中缓存、英文整句可提交。费用账本正确记录 100 输入、20 输出、40 缓存输入 token；自定义服务的价格保持未知。费用窗口已打开并检查画面，菜单摘要经真实协议返回；未安装 TSF，不能把这等同于真实任务栏菜单验收。模拟测试未调用外部服务。

新设置程序已完成编译和代码测试，但启动离线截图模式时被本机 Windows 应用控制拦截（4551）；没有关闭系统安全策略。设置窗口的运行验收仍待测试电脑完成。

补充的私有 Actions 验证已上传包含 83 个差异文件的源码快照，提交 `b52537dc99cddb750dc6d705cc9531f00434ee38`，运行地址为 https://github.com/boluomi2001/qingjian-bilingual-validation/actions/runs/39281369725 。打包前状态仍为排队，未计为通过；此快照覆盖最终运行代码，之后仅调整交付文档和打包脚本。

这些结果不代表已完成 Windows 真机应用验收。尚未验证真实输入框中的按键与上屏，也未使用付费 API；详见下节“真机边界”。

## 真机边界

尚未在开发电脑注册或安装此版输入法。真正的 TSF 按键回调、常用应用中的上屏、安装和卸载仍须按《双语测试验收》在测试电脑复测。未调用付费模型，也未读取用户的 DeepSeek 密钥或账单；模型质量和平台实际费用继续由用户实测核对。
