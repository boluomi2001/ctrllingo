# Cloudflare Pages 部署记录

状态核对日期：2026-10-24。以下记录已核实的 Pages 配置和域名状态。

## Pages 项目

- Pages 项目：`ctrllingo-website`，已创建并连接 GitHub 仓库 `boluomi2001/ctrllingo`。
- 生产分支：`website`；Root directory：`website/`。Cloudflare 的源码路径过滤为 `website/*`。
- Production 自动部署已启用；Preview 部署面向所有分支并受相同路径过滤。已启用 `website` 分支的 `website/*` 推送自动构建；每次构建和正式版本切换结果在 Pages 部署历史核对。
- 构建命令：`pnpm run build`；输出目录：`dist`。
- 构建环境：`NODE_VERSION=24.22.1`、`PNPM_VERSION=11.25.0`、`NPM_CONFIG_REGISTRY=https://registry.npmjs.org`。
- 首次 Git 构建成功，部署 ID：`d1fb3961-0abf-4376-870b-2904c601e255`；产物 25 个文件，`uses_functions=false`。
- 首次文档推送创建的记录 `2d5650ea-578c-429c-8853-966b3c8f81a9` 虽显示 queued，但 `is_skipped=true`，不能视为已开始构建。随后将文件监听规则改为官方示例格式 `website/*`；自动发布结果须同时核对 `is_skipped=false`、最终 deploy success 和正式版本已切换，不能只看记录已创建。
- Cloudflare 预览地址：<https://ctrllingo-website.pages.dev/>。

## 正式域名

- `https://liyuxing.icu/` 与 `https://www.liyuxing.icu/` 均已绑定 Pages，状态为 active；证书状态为 active，实际 HTTPS 请求均返回 200。
- `www` 直接提供相同站点内容；页面 canonical 指向 `https://liyuxing.icu/`。没有新增 www 重定向规则。
- 域名体系中的 `sph.liyuxing.icu` 用于视频业务；现有精确 route `sph.liyuxing.icu/* → sph-parse` 与 `AAAA 100::` 记录保持原样。
- 本次没有修改整区 nameserver、SSL 模式（Full）、Always Use HTTPS、最低 TLS 版本（TLS 1.0）或通配符配置。

## 视频业务基线

- 部署前后核对中，视频 API 返回 HTTP 200、`errCode=0`。
- 原 `videoUrl` 直链在部署前后均可做 HTTP Range 请求并返回 206；读取的 1024 字节摘要一致。
- `originVideoUrl` 字段在部署前后均返回 HTTP 400，这是既有问题，不能归因于本次部署。
- 正式部署前曾观察到上游 `parseShareUrl` 返回 401；随后该问题在正式部署前已恢复。当前观察不证明上游问题已永久解决。

## 变更与回滚边界

官网发布源是同仓库 `website` 分支的 `website/` 目录；更新步骤见 [`DEPLOYMENT.md`](DEPLOYMENT.md)。不要把官网变更提交到维护输入法软件的 `main` 分支。

若本轮 Pages 版本导致问题，只回滚 Pages 到此前部署版本；若确认是本轮为 apex 或 www 新增的绑定 / DNS 变更所致，只回退对应的 apex / www 项目级变更。回滚不得覆盖整区 DNS，也不得修改 `sph.liyuxing.icu`、其 route 或 `AAAA 100::`、nameserver、SSL 或通配符配置。

本文件不记录 Cloudflare 账号 ID、DNS 记录 ID、Cookie、API 密钥或带令牌的视频 URL。
