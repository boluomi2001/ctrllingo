---
title: 安装
order: 1
description: CtrlLingo 译点 Windows 0.1.0 预发布版的系统目标、安装、使用和升级说明。
---

## macOS

CtrlLingo 0.1.0 不发布 macOS 安装包。仓库中保留的 macOS 内容属于 Qingjian 上游继承资料，不代表译点提供或验收了 macOS 版本；请勿按上游文件名安装译点。

## Windows

**系统目标**：Windows 10 版本 1809 或更新版本，以及 Windows 11，均为 x64。当前尚未完成所有目标系统和应用的真机验收。

1. 在 [CtrlLingo GitHub Releases](https://github.com/boluomi2001/ctrllingo/releases) 下载 `CtrlLingo-0.1.0-windows-x64-setup.exe`。核对发布页提供的 `SHA256SUMS.txt`。
2. 运行安装包并按提示授予管理员权限。默认安装位置为 `%ProgramFiles%\CtrlLingo`；程序没有代码签名，Windows 可能提示发布者未知或显示安全警告。请先核对来源和校验值，不要关闭 Windows 安全保护或绕过保护运行文件。
3. 安装完成后按 `Win + Space`，在 Windows 输入法列表中选择「译点 CtrlLingo」。若应用在安装前已打开，请重新打开该应用，或注销后重新登录。

安装器注册 x64 TSF 供 64 位应用使用，也注册 x86 TSF 供 32 位应用使用。设置程序可从开始菜单的「CtrlLingo 设置」打开；系统托盘显示「译点 CtrlLingo」。首次使用可在设置中选择学习语言。按需翻译需在「云服务」页配置兼容 API 地址与密钥。

**候选窗口限制**：安装目录在 `%ProgramFiles%` 不会为未签名程序启用 `uiAccess`。当前 Server 使用 `uiAccess=false`，因此任务栏搜索、开始菜单等部分系统界面可能遮挡候选窗口；请勿通过关闭系统保护来处理。

## 升级与卸载

0.1.0 以预发布版提供，直到原生安装、升级、卸载和真实 TSF 上屏验收完成。升级可运行新版本安装包；正在运行的 Server 和设置程序会由安装器处理。已打开的宿主应用可能仍加载旧 TSF DLL，重新打开应用或注销后才会使用新版本。用户配置和学习数据会保留。

卸载步骤见[卸载](../help/uninstall.md)。
