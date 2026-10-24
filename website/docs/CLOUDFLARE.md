# Cloudflare Pages 准备配置（待执行）

此文件记录部署前的配置方案，不代表 Pages 项目已创建、域名已绑定或网站已部署。

## Pages 配置候选

- GitHub 仓库：`boluomi2001/ctrllingo`
- Pages 项目候选名：`ctrllingo-website`
- 生产分支：`website`
- Root directory：`website`
- Build command：`pnpm run build`
- Build output directory：`dist`
- Node.js：`24.22.1`（`.node-version`）
- `PNPM_VERSION`：`11.25.0`
- 可设置 `NPM_CONFIG_REGISTRY=https://registry.npmjs.org` 覆盖本机 `.npmrc` 中的镜像地址。
- 仅监测 `website/**` 的变更。

## 域名与现有业务约束

- Pages 仅计划绑定 `liyuxing.icu` 与 `www.liyuxing.icu`。
- 保留 `sph.liyuxing.icu` 当前精确 route 到 `sph-parse`，及其 `AAAA 100::` 记录；不得修改或覆盖。
- 不修改 nameserver（NS）、SSL 设置或通配符路由/记录。
- 域名体系中的 `sph.liyuxing.icu` 用于视频业务；apex `liyuxing.icu` 与 `www.liyuxing.icu` 当前未配置 DNS 记录。
- 部署前，用户提供的两条视频链接请求均返回 HTTP 500；上游 `parseShareUrl` 返回 HTTP 401。直链与 HTTP Range 范围尚未测试，不能表述为已通过。
- 此方案不代表已创建、已绑定或已部署，也不构成对现有视频业务影响的验证结论。

本文件不记录账号 ID、Cookie、密钥或其他凭据。
