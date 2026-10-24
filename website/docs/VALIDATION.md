# 官网部署与验证记录

实测日期：2026-10-24。本文区分已观察到的结果与尚未覆盖的范围；浏览器回归不代表实体设备、所有地区网络或长期服务稳定性已得到验证。

## Cloudflare Pages 部署

- Pages 项目 `ctrllingo-website` 已连接 `boluomi2001/ctrllingo` 的 `website` 分支，Root directory 为 `website/`，仅监测 `website/*`。
- 首次 Git 构建成功，部署 ID 为 `d1fb3961-0abf-4376-870b-2904c601e255`；输出 25 个文件，`uses_functions=false`。
- 首次文档推送创建的记录 `2d5650ea-578c-429c-8853-966b3c8f81a9` 虽显示 queued，但 `is_skipped=true`，不能视为已开始构建。随后将文件监听规则改为官方示例格式 `website/*`；自动发布结果须同时核对 `is_skipped=false`、最终 deploy success 和正式版本已切换，不能只看记录已创建。
- `https://liyuxing.icu/` 与 `https://www.liyuxing.icu/` 均为 active 绑定，证书 active，实际 HTTPS 返回 200。www 直接提供相同站点，canonical 指向 apex，没有新增重定向规则。
- `https://ctrllingo-website.pages.dev/` 可作为预览地址。
- 预览与正式地址均在实际 Edge / Playwright 的 1440×1000 与 390×844 视口完成回归：页面交互通过，未记录 console error、pageerror、HTTP 4xx/5xx 资源或失败请求。
- 页面实际包含唯一 canonical `https://liyuxing.icu/`、同址 `og:url`、`robots.txt` 和只列首页的 `sitemap.xml`。三个“立即下载”锚点的 href 均指向已发布的 CtrlLingo 0.1.1 Windows x64 安装器；本项核对链接目标，不代表本轮重新完整下载了安装器。
- 生产 HTML 被 Cloudflare 自动插入与部署前视频站同一站点 ID 的统计 beacon。其余 24 个静态文件逐字节一致；剔除已知 beacon 和标签间空白后，生产 HTML 与构建产物一致。本轮未调整分析或安全策略。
- 默认 Python User-Agent 请求曾返回 403；实际 Edge / 浏览器 User-Agent 请求返回 200。线上验收以实际浏览器结果为准，本轮没有调整全局安全策略。

## 站点交互与视觉检查

- 早期本地浏览器回归覆盖 1920、1440、1280、1024、768、390、375px；页面无横向溢出、文字裁切、缺失锚点或导航碰撞。正式域名和 Pages 预览的本轮回归覆盖 1440×1000 与 390×844。
- 桌面五个导航链接更新 hash 并将焦点移至对应标题；手机菜单 tap、Escape、FAQ 跳转、关闭状态与焦点返回通过。
- Hero 的 Ctrl 翻译、Space 确认和重播通过；FAQ 第二项可切换；系统 reduced-motion 下装饰动效关闭，演示按钮仍可用。
- 页面 lang、单一 H1、canonical、页面图片加载与资源响应通过本轮检查。
- 下载按钮仅核对 href，不触发约 160 MB 安装包下载。
- 截图、修复记录与其他阶段交付材料仍保留在各阶段验收目录中。

## sph 视频业务前后基线

- `sph.liyuxing.icu` 的现有精确 route `sph.liyuxing.icu/* → sph-parse` 与 `AAAA 100::` 记录保持原样；本次没有调整整区 nameserver、SSL Full / Always Use HTTPS / 最低 TLS 1.0 或通配符配置。
- 视频 API 观察结果为 HTTP 200、`errCode=0`。原 `videoUrl` 直链在部署前后 Range 请求均返回 206，读取的 1024 字节摘要一致。
- `originVideoUrl` 在部署前后均返回 HTTP 400，属于已存在问题，不能归因于本次部署，也不能说该字段可用。
- 正式部署前曾观察到上游 `parseShareUrl` 401，随后在正式部署前已恢复。这个时间点的恢复不证明上游问题已永久解决。

## 尚未覆盖

- 实体 iPhone / Android、Safari、Firefox、辅助阅读器、系统缩放与文字放大。
- 线上 Core Web Vitals、真实访客性能、搜索引擎收录与社交平台卡片的长期结果。
- 完整回滚演练。
- 视频上游的长期可用性，以及 `originVideoUrl` HTTP 400 问题的修复。
- 浏览器链接检查未覆盖安装器的全量下载耗时与最终用户网络完整性。

## 许可与真实性边界

截图、录屏不代表网站连接了真实输入法。用量截图不是实时账户余额或真实账单验证。输入法与网站许可、品牌及媒体授权边界见 README 和软件仓库许可文件；本次部署和文档更新不改变这些许可。
