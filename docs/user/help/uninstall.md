---
title: 卸载
order: 2
description: 卸载 CtrlLingo 译点并了解卸载后保留的个人数据。
---

## macOS

CtrlLingo 0.1.0 不发布 macOS 版本。仓库中保留的 macOS 卸载步骤属于 Qingjian 上游继承资料，不适用于译点 Windows 安装包。

## Windows

从开始菜单打开「卸载 CtrlLingo」，或在 Windows「设置 → 应用 → 已安装的应用」中选择 CtrlLingo 并卸载。卸载需要管理员权限，并移除程序及 64 位、32 位 TSF 注册。

宿主应用可能仍在内存中加载 TSF DLL。卸载后关闭并重新打开这些应用，必要时注销后重新登录，旧 DLL 才会完全退出。

卸载会保留个人配置、API 密钥、学习数据、统计和运行日志：

- `%APPDATA%\CtrlLingo`：配置和个人数据。
- `%LOCALAPPDATA%\CtrlLingo`：运行日志。

如需一并删除这些个人文件，请在卸载后手动删除上述文件夹。此操作不可恢复。云服务商已收到的请求不受本地卸载影响；其保存规则由服务商决定。

## 只清除一部分

- 只清除输入日志：「设置 → 高级 → 清空输入日志」。
- 删除某个学到的词：输入拼音时按 `⇧ + 数字`，见 [译词与生词](../learning/translation.md#删除不需要的候选)。
- 移除导入的词库：「设置 → 词库 → 移除」只将文件移到数据目录的 `dicts/removed/`，需彻底删除时自行清理该目录。
