# CtrlLingo · 译点官网

AI 多语言输入助手的单页官网与个人项目展示。阶段 04 已完成本地视觉回归和发布材料准备；最终视觉效果由项目方验收，尚未部署到公网。

导航、Hero、产品价值、Bento 功能、三步流程、使用场景、真实软件预览、设计故事、开源说明、FAQ、底部 CTA 和页脚共用同一套响应式组件。Hero 是预设文本交互演示，不连接 AI API 或真实输入法；产品预览中的截图和约 47 秒录屏由项目方提供。

## 环境

- 已验证：Windows，Node.js 24.22.1，pnpm 11.25.0。
- `package.json` 要求 Node.js 24+，并记录包管理器版本；实际依赖由 `pnpm-lock.yaml` 固定。
- 官网发布拟复用同一 GitHub 仓库的 `website` 分支 `website/` 目录；`main` 分支继续维护输入法软件，原始本地官网目录 `ctrllingo-web` 保持独立。此说明记录的是发布方案，不代表内容已推送或上线。

## 本地运行

已有依赖时，Windows 可双击 `启动官网.cmd`。优先使用 5173，端口占用时自动选择可用端口，并打开对应地址；以窗口输出为准。页面已打开时可直接刷新。关闭该服务窗口可停止对应预览。

其他环境先安装 Node.js 和 pnpm，再在本目录执行：

```bash
pnpm install --frozen-lockfile
pnpm run dev
```

在输出的 Local 地址打开。此项目采用 pnpm；不要混用锁文件。项目内 `.npmrc` 使用 npm 镜像，`pnpm-workspace.yaml` 将依赖存储放在项目目录。下载依赖需要访问配置的包仓库。

## 检查与构建

```bash
pnpm run check
pnpm run build
pnpm run preview
```

构建输出为 `dist/`；`preview` 只用于本地检查构建产物。线上请将 `dist/` 交给静态托管服务，不要将开发服务器作为公网生产服务，也不要直接双击 `dist/index.html`。

## 结构

- `src/App.tsx`：导航、Hero 文案、安装包下载入口和键盘跳过导航。
- `src/HeroDemo.tsx`：预设翻译演示、Ctrl / Space 操作与装饰暂停。
- `src/HomeSections.tsx`：价值、功能、流程、场景、截图 / 录屏、故事、开源与 FAQ。
- `src/HomeFooter.tsx`：底部 CTA、项目与许可链接、动态年份。
- `src/design-tokens.css`：颜色、圆角、阴影、模块间距。
- `src/usePageMotion.ts`、`src/usePointerEffects.ts`：滚动入场、错开、指针柔光和磁吸。
- `public/media/`：正式 PNG 母版、网页 WebP、图标与录屏。
- `licenses/`、`public/third-party-notices.txt`：选定网站依赖的许可原文和范围说明。
- `docs/DEPLOYMENT.md`：部署参数、域名与发布清单。
- `docs/VALIDATION.md`：本轮验证范围与未覆盖项。

## 素材和交互

保留奶油白 / 鼠尾草绿 Design Tokens、正式 CtrlDot Logo、Ctrl / Space 键帽、玻璃背景，以及项目方确认的《梦幻薄雾青山全景.png》。原始 PNG 不覆盖，网页使用相同比例的优化派生文件；真实截图仍为原 PNG。

录屏保留 1198×496、30fps、约 46.87 秒、1406 帧。网页副本约 0.28 MB，原视频的全静音音轨已移除；封面来自第 39 秒。默认不自动播放、不循环，点击才加载。建议全屏观看，手机可横屏查看。

滚动入场在每个目标首次进入视口时播放一次，刷新可重看。精确鼠标效果只在宽度至少 1024px、支持 hover / fine pointer 且系统允许动效时启用。手机使用同一内容，保留滚动和点击；系统 reduced-motion 下关闭装饰动效。演示快捷键仅在演示区本身获得焦点时响应。

## 真实性与许可边界

页面中的软件截图、录屏不代表当前网页接入真实输入法。关于页截图显示 0.1.0；用量截图中的金额是原截图内容，仅作界面说明，不是网站实时余额或真实账单验证。

页首、首屏和手机菜单中的“立即下载”直接指向已核实的 CtrlLingo 0.1.1 Windows x64 安装器；下载由浏览器处理，页面不会运行安装器。底部仍保留 GitHub 发布说明与开源许可入口。首次翻译需要配置服务地址与 API 密钥，服务商可能收费。原生安装、更多应用 / DPI、真实账户账单等未覆盖范围，以软件仓库的发布验收记录为准。

输入法基于 Qingjian 独立二次开发，采用 GPL-3.0-or-later；数据和部分组件保留各自许可。网站的 React / 图标 / 构建工具许可说明不等于给品牌图、文案或媒体重新授权；官网独立源码对外发布的许可选择仍由项目方决定。

不在源码、前端环境变量或静态托管中放入真实翻译 API 密钥。此官网无需后台账户、数据库或翻译服务密钥。

## 当前验收状态

已验证与待测项目见 `docs/VALIDATION.md`。完整截图、修复前后记录与验收报告另附于阶段 04 交付目录。实体手机、Safari / Firefox、辅助阅读器、线上 CDN 与真实用户性能仍需进一步测试。
