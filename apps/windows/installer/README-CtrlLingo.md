# CtrlLingo · 译点 Windows 安装包

本目录的独立安装器只注册和维护 CtrlLingo TSF。安装位置默认为 `%ProgramFiles%\CtrlLingo`；输入法用户配置和学习数据位于 `%APPDATA%\CtrlLingo`，卸载不会删除该目录。机器级 TSF 图标由注册时写入 `%ProgramData%\CtrlLingo\ctrllingo.ico`。

## 构建

在 Windows PowerShell 5.1 或 PowerShell 7 中，从仓库根目录运行：

```powershell
$env:CTRLLINGO_ISCC = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
.\apps\windows\installer\build-ctrllingo.ps1
```

实际脚本路径为 `apps\windows\installer\build-ctrllingo.ps1`。默认会用 MSVC 工具链锁定依赖构建 64 位 Server、设置程序、TSF DLL，以及 32 位 TSF DLL，然后按 `tools\release\data.lock` 获取并校验产品数据。安装器、源代码快照和 SHA-256 清单写入 `target\ctrllingo-release`。自定义 `-OutputDirectory` 必须仍在仓库 `target` 目录内。

本机 GNU 构建可用 `-SkipBuild -BinaryDirectory <64位产物目录> -BinaryDirectory32 <32位DLL目录>` 将已构建文件交给打包器；`-Target`、`-Target32`、`-OutputDirectory` 可调整目标与输出目录。打包会生成 `CtrlLingo-0.1.1-windows-x64-setup.exe` 与独立的 `CtrlLingo-0.1.1-source.zip`，两者应配套分发。源快照包含 Git 已跟踪文件和未忽略的未跟踪文件；Git 忽略的 `target/`、`.env` 等不会进入快照。

发布包携带项目 `LICENSE`、`NOTICE`、隐私说明、验证说明、Cargo 依赖许可证清单与许可证原文，以及 `assets/` 下 README 和许可证原文。产品数据只能从已锁定并验过 SHA-256 的数据包提取；获取脚本还会把验证后的词库和模型同步到仓库 `data/generated` 与 `data/models`，供本地运行使用。

## 安装与卸载行为

- 安装器以管理员权限注册 64 位与 32 位 TSF，并给安装目录授予应用容器只读执行权限。
- TSF DLL 按版本并排保存。升级时关闭 CtrlLingo Server 和设置程序，再注册新 DLL；已经打开的应用会继续使用它已加载的 DLL，重新打开应用或注销后才会加载新版本。
- 登录启动项由 Explorer 以当前用户身份启动 CtrlLingo Server。安装完成页也会尝试用原用户身份启动 Server。
- 安装时只在 `%APPDATA%\CtrlLingo\config.toml` 不存在时写入随包默认配置。卸载保留用户配置、密钥、学习记录和统计。
- 安装器不签名，Server 的 `uiAccess` 关闭。输入法可在普通桌面应用中使用；候选窗在任务栏搜索及部分系统界面中可能被其他窗口遮挡。此限制来自 Windows 对未签名 `uiAccess` 程序的启动与显示规则。请保持系统安全防护开启。

该安装器只生成安装介质，不会在构建机上注册输入法或实际安装产品。发布前的真机验证由维护者按包内验证说明执行。
