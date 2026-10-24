# Cloudflare Pages 部署与更新

状态核对日期：2026-10-24。本官网已部署到 Cloudflare Pages。正式配置、域名约束和视频业务基线见 [`CLOUDFLARE.md`](CLOUDFLARE.md)；实际检查范围见 [`VALIDATION.md`](VALIDATION.md)。

## 当前部署配置

| 项目 | 已配置值 |
| --- | --- |
| Cloudflare Pages 项目 | `ctrllingo-website` |
| GitHub 仓库 | `boluomi2001/ctrllingo` |
| 生产分支 | `website` |
| Root directory | `website/` |
| 源码路径过滤 | `website/*` |
| 构建命令 | `pnpm run build` |
| 输出目录 | `dist` |
| Node | `24.22.1` |
| pnpm | `11.25.0` |
| NPM registry | `https://registry.npmjs.org` |
| 正式地址 | `https://liyuxing.icu/`、`https://www.liyuxing.icu/` |
| 预览地址 | `https://ctrllingo-website.pages.dev/` |

Production 自动部署已启用，Preview 面向所有分支；二者均只对 `website/*` 路径变更构建。已启用 `website` 分支的 `website/*` 推送自动构建；每次构建和正式版本切换结果在 Pages 部署历史核对。首次 Git 构建已成功，部署 ID 和实际域名结果见 `CLOUDFLARE.md`。

这是静态 Vite 站点，不需要前端服务密钥、后台账户或数据库。环境变量设置 `NPM_CONFIG_REGISTRY=https://registry.npmjs.org`，以覆盖本机 `.npmrc` 中的镜像地址。网页的安装包按钮由浏览器直连 GitHub Release，不执行安装器。

## 在同一仓库更新官网

官网线上源码在 `website` 分支的 `website/` 目录。原始本地开发副本 `ctrllingo-web` 与该 Git 工作区独立；需要更新线上内容时，应在仓库的 `website` 分支工作区修改 `website/` 中的文件。

可从仓库根目录运行：

```bash
git clone --branch website https://github.com/boluomi2001/ctrllingo.git ctrllingo-site
cd ctrllingo-site/website
```

在 `website/` 中编辑和检查后，回到仓库根目录提交并推送到 `website`：

```bash
cd ..
git add website
git commit -m "Update CtrlLingo website"
git push origin website
```

只把官网文件加入 `website/`。`main` 分支继续维护输入法软件，不要把官网改动提交到 `main`。已确认 `website/*` 推送会自动创建 Pages 构建；推送后应在 Pages 部署历史核对构建结果和正式版本状态。

## 域名、资源和回滚

页面资源使用 `/media/...` 等根路径，站点应发布在域名根目录。正式 canonical 与 Open Graph URL 使用 `https://liyuxing.icu/`；www 直接提供同一站点，不新增重定向规则。`robots.txt` 与 `sitemap.xml` 只声明主页。

仅处理本轮官网的 Pages 版本及 apex / www 绑定或 DNS 变更。若需回退，优先恢复此前可用的 Pages 部署；只有确认问题来自新增的 apex / www 绑定或 DNS 时，才回退相应项目级变更。不得覆盖整区 DNS，不得更改 `sph.liyuxing.icu` 的精确 route 或 `AAAA 100::`，也不更改 nameserver、SSL 或通配符配置。

现有视频服务的实测基线和问题范围见 `CLOUDFLARE.md`。特别是 `originVideoUrl` 的既有 HTTP 400 与短暂恢复的上游 401，不应写成已永久解决。

## 下载与许可边界

页首、首屏和手机菜单中的三个“立即下载”入口直连已核对的 CtrlLingo 0.1.1 Windows x64 安装器。浏览器负责下载；官网不运行安装器。升级前应核对软件版本、对应源码、LICENSE / NOTICE、第三方许可和校验材料。

输入法基于 Qingjian 独立二次开发并采用 GPL-3.0-or-later。官网依赖许可不自动授权品牌图、文案或媒体；源码对外发布的许可选择仍由项目方决定。此次文档更新不改变既有许可边界。
