import { useRef, useState } from 'react'
import {
  AppWindow,
  ArrowUpRight,
  BookOpen,
  BriefcaseBusiness,
  ChartNoAxesCombined,
  ChevronDown,
  PenLine,
  ToggleRight,
  Wallet,
  Zap,
} from 'lucide-react'
import './home-sections.css'

const features = [
  {
    number: '01',
    title: 'Ctrl 按需翻译',
    description: '核对当前中文候选，单独按下并松开 Ctrl。已有译文可复用，未命中时再请求翻译服务。',
    image: '/media/ctrl-key-512.webp',
    imageAlt: 'Ctrl 键帽',
    className: 'home-feature-ctrl',
  },
  {
    number: '02',
    title: 'Space 译文上屏',
    description: '译文就绪后按 Space，将当前选中的译文提交到输入位置。翻译等待时保留当前输入。',
    image: '/media/space-key-512.webp',
    imageAlt: 'Space 键帽',
    className: 'home-feature-space',
  },
  {
    number: '03',
    title: '五种目标语言',
    description: '中文可转换为英语、日语、德语、西班牙语和越南语；一次选择一种目标语言。',
    className: 'home-feature-languages',
  },
  {
    number: '04',
    title: '用量与费用可见',
    description: '查看输入 / 输出 Token、费用估算及支持服务的账户余额；实际费用以服务平台账单为准。',
    className: 'home-feature-usage',
  },
]

const faq = [
  {
    question: 'CtrlLingo 是翻译软件还是输入法？',
    answer: '它是基于青简二次开发的 AI 多语言输入助手，把中文输入、按需翻译和译文上屏接在同一个输入流程中。普通中文输入与翻译功能按配置使用。',
  },
  {
    question: '支持哪些语言？',
    answer: '当前二次开发版本支持将中文转换为英语、日语、德语、西班牙语和越南语。一次选择一种目标语言；不代表五种语言同时输出，也不代表已完成专业翻译质量评测。',
  },
  {
    question: '会自动翻译我输入的所有内容吗？',
    answer: 'Ctrl 整句翻译由你按需触发。云联想另有独立设置；启用哪些云功能会影响请求行为。请求翻译时，当前句子会发送到配置的翻译服务，具体使用方式见版本说明。',
  },
  {
    question: '使用 AI 翻译会产生费用吗？',
    answer: '可能产生，取决于配置的服务、模型和实际用量。本地面板提供用量与费用估算；未知价格或缺失用量会如实提示，不能保证完全等于平台账单。当前账户余额查询面向 DeepSeek 官方服务，其他服务未必支持。官网不显示虚构的实际费用或售价。',
  },
  {
    question: '目前支持什么操作系统？',
    answer: '本项目当前提供 Windows x64 版本，发布说明与验收范围见项目仓库。上游具有其他平台代码，不代表本项目的 Ctrl 翻译、费用面板等改动已在其他系统完成适配和验收。手机版网页只是官网响应式展示，不是手机输入法产品。',
  },
]

const screenshots = [
  {
    src: '/media/product-usage.png',
    alt: 'CtrlLingo 用量面板截图，含五种语言下拉菜单',
    title: '用量与费用窗口',
    caption: '软件界面截图 · 数值为截取时点，仅作界面展示',
    className: 'home-preview-usage',
    width: 449,
    height: 535,
  },
  {
    src: '/media/product-settings.png',
    alt: 'CtrlLingo 设置程序关于页，显示版本 0.1.0',
    title: '设置程序关于页',
    caption: '设置关于页 · 截图显示版本 0.1.0',
    className: 'home-preview-settings',
    width: 1125,
    height: 889,
  },
  {
    src: '/media/product-candidates.png',
    alt: '中文候选状态的候选窗口截图',
    title: '中文候选',
    caption: '候选窗口截图 · 中文与译文对照',
    className: 'home-preview-candidate',
    width: 300,
    height: 251,
  },
  {
    src: '/media/product-translating.png',
    alt: '翻译等待且暂无译文的候选窗口截图',
    title: '翻译等待',
    caption: '候选窗口截图 · 中文与译文对照',
    className: 'home-preview-candidate',
    width: 314,
    height: 259,
  },
  {
    src: '/media/product-translated.png',
    alt: '显示 You look so beautiful today! 译文的候选窗口截图',
    title: '译文就绪',
    caption: '候选窗口截图 · 中文与译文对照',
    className: 'home-preview-candidate',
    width: 419,
    height: 259,
  },
] as const

const candidateScreenshots = [screenshots[2], screenshots[3], screenshots[4]] as const

function HomeSections() {
  const [image, setImage] = useState<(typeof screenshots)[number] | null>(null)
  const [candidateImage, setCandidateImage] = useState<(typeof candidateScreenshots)[number]>(candidateScreenshots[2])
  const dialogRef = useRef<HTMLDialogElement>(null)
  const openerRef = useRef<HTMLButtonElement>(null)

  const showImage = (item: (typeof screenshots)[number], opener: HTMLButtonElement) => {
    openerRef.current = opener
    setImage(item)
    dialogRef.current?.showModal()
  }

  return (
    <div className="home-sections">
      <section className="home-section" id="why" aria-labelledby="home-why-title">
        <div className="home-container">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">WHY CTRLLINGO</p>
            <h2 id="home-why-title">为什么是 CtrlLingo？</h2>
            <p>把按需翻译接入输入过程，让表达少一步切换。</p>
          </header>
          <div className="home-why-grid">
            <article className="home-card home-why-card" data-motion="enter" data-motion-delay="0" data-pointer="card">
              <span className="home-icon-tile"><AppWindow size={25} aria-hidden="true" /></span>
              <h3>不离开当前输入位置</h3>
              <p>在当前输入流程中组织中文，再按需获取目标语言译文。</p>
            </article>
            <article className="home-card home-why-card" data-motion="enter" data-motion-delay="80" data-pointer="card">
              <span className="home-icon-tile"><Zap size={25} aria-hidden="true" /></span>
              <h3>保持表达节奏</h3>
              <p>译文就绪后按 Space 提交，减少在输入与翻译之间切换。</p>
            </article>
            <article className="home-card home-why-card" data-motion="enter" data-motion-delay="160" data-pointer="card">
              <span className="home-icon-tile"><ToggleRight size={25} aria-hidden="true" /></span>
              <h3>由你决定何时翻译</h3>
              <p>核对当前中文候选后，单独按下并松开 Ctrl 触发翻译。</p>
            </article>
          </div>
        </div>
      </section>

      <section className="home-section" id="features" aria-labelledby="home-features-title">
        <div className="home-container">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">CORE FEATURES</p>
            <h2 id="home-features-title">核心功能</h2>
          </header>
          <div className="home-feature-grid">
            {features.map((feature, index) => (
              <article className={`home-card home-feature-card ${feature.className}`} key={feature.number} data-motion="enter" data-motion-delay={index * 90} data-pointer="card">
                <div className="home-feature-topline"><span>{feature.number}</span>
                  {feature.image ? <img src={feature.image} alt={feature.imageAlt} width="512" height="512" decoding="async" loading="lazy" /> : feature.number === '03' ? <BookOpen size={25} aria-hidden="true" /> : <Wallet size={25} aria-hidden="true" />}
                </div>
                <h3>{feature.title}</h3>
                <p>{feature.description}</p>
                {feature.number === '03' && <ul className="home-language-list"><li>English</li><li>日本語</li><li>Deutsch</li><li>Español</li><li>Tiếng Việt</li></ul>}
                {feature.number === '04' && <div className="home-usage-icons" aria-hidden="true"><ChartNoAxesCombined size={21} /><Wallet size={21} /></div>}
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="home-section" id="how-it-works" aria-labelledby="home-how-title">
        <div className="home-container home-how-panel">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">HOW IT WORKS</p>
            <h2 id="home-how-title">三步完成多语言输入</h2>
          </header>
          <ol className="home-steps">
            <li data-motion="step" data-motion-delay="0" data-pointer="card"><span className="home-step-number">01</span><h3>输入中文</h3><p>用熟悉的拼音输入方式组织完整句子，核对当前高亮候选。</p><span className="home-step-connector" aria-hidden="true"><span className="home-step-link-progress" /></span></li>
            <li data-motion="step" data-motion-delay="180" data-pointer="card"><span className="home-step-number">02</span><h3>按下 Ctrl</h3><p>单独按下并松开 Ctrl，按需获取目标语言译文；等待完成。</p><span className="home-step-connector" aria-hidden="true"><span className="home-step-link-progress" /></span></li>
            <li data-motion="step" data-motion-delay="360" data-pointer="card"><span className="home-step-number">03</span><h3>按下 Space</h3><p>核对译文后按 Space，提交到当前输入位置。</p></li>
          </ol>
          <div className="home-flow-example" data-motion="enter" data-motion-delay="360" data-pointer="card">
            <span>流程示意 · 预设文本</span>
            <p>我明天要去开会。 <span aria-hidden="true">→</span> I have a meeting tomorrow.</p>
          </div>
          <p className="home-flow-notes" data-motion="enter" data-motion-delay="420">翻译等待时保留当前输入；失败后可按 Ctrl 重试；编辑后旧回复失效。Ctrl+C / Ctrl+V 等组合键保持原用途。不能回收任意已提交的中文再翻译。</p>
          <a className="home-text-link" href="#demo" data-motion="enter" data-motion-delay="460">查看交互演示 <span aria-hidden="true">→</span></a>
        </div>
      </section>

      <section className="home-section" id="scenarios" aria-labelledby="home-scenarios-title">
        <div className="home-container">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">SCENARIOS</p>
            <h2 id="home-scenarios-title">适用场景</h2>
          </header>
          <div className="home-scenario-grid">
            <article className="home-card home-scenario-card" data-motion="enter" data-motion-delay="0" data-pointer="card"><BriefcaseBusiness size={27} aria-hidden="true" /><h3>工作沟通</h3><p>在写邮件或消息时，用中文组织表达，再按需输出目标语言。具体应用支持情况以版本验证记录为准。</p></article>
            <article className="home-card home-scenario-card" data-motion="enter" data-motion-delay="90" data-pointer="card"><BookOpen size={27} aria-hidden="true" /><h3>语言学习</h3><p>对照中文与译文，辅助理解常用表达；不承诺教学效果或替代专业校对。</p></article>
            <article className="home-card home-scenario-card" data-motion="enter" data-motion-delay="180" data-pointer="card"><PenLine size={27} aria-hidden="true" /><h3>多语言内容创作</h3><p>为简介、说明和短内容提供翻译辅助；重要内容仍需核对语境与专有名词。</p></article>
          </div>
        </div>
      </section>

      <section className="home-section" id="preview" aria-labelledby="home-preview-title">
        <div className="home-container">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">PRODUCT PREVIEW</p>
            <h2 id="home-preview-title">产品界面预览</h2>
          </header>
          <div className="home-preview-grid">
            {screenshots.slice(0, 2).map((item, index) => (
              <figure className={`home-card home-preview-card ${item.className}`} key={item.src} data-motion="enter" data-motion-delay={index * 90} data-pointer="card">
                <button className="home-preview-image-button" type="button" onClick={(event) => showImage(item, event.currentTarget)} aria-label={`查看大图：${item.title}`}>
                  <img src={item.src} alt={item.alt} width={item.width} height={item.height} decoding="async" loading="lazy" />
                </button>
                <figcaption><span><strong>{item.title}</strong><small>{item.caption}</small></span><button className="home-preview-open" type="button" onClick={(event) => showImage(item, event.currentTarget)}>查看大图</button></figcaption>
              </figure>
            ))}
            <figure className="home-card home-preview-card home-preview-candidate-card" data-motion="enter" data-motion-delay="180" data-pointer="card">
              <button className="home-preview-image-button home-preview-candidate-stage" type="button" onClick={(event) => showImage(candidateImage, event.currentTarget)} aria-label={`查看大图：${candidateImage.title}`}>
                <img src={candidateImage.src} alt={candidateImage.alt} width={candidateImage.width} height={candidateImage.height} decoding="async" loading="lazy" />
              </button>
              <div className="home-preview-candidate-selector" role="group" aria-label="选择候选窗口状态">
                {candidateScreenshots.map((item) => (
                  <button key={item.src} type="button" aria-pressed={candidateImage.src === item.src} onClick={() => setCandidateImage(item)}>{item.title}</button>
                ))}
              </div>
              <figcaption><span><strong>{candidateImage.title}</strong><small>{candidateImage.caption}</small></span><button className="home-preview-open" type="button" onClick={(event) => showImage(candidateImage, event.currentTarget)}>查看大图</button></figcaption>
            </figure>
            <figure className="home-card home-preview-card home-preview-video-card" data-motion="enter" data-motion-delay="270">
              <video src="/media/product-demo.mp4" poster="/media/product-demo-poster.webp" controls playsInline preload="none" width={1198} height={496} aria-label="CtrlLingo 实际软件输入与翻译录屏" aria-describedby="product-demo-caption">
                <a href="/media/product-demo.mp4">打开演示视频</a>
              </video>
              <figcaption id="product-demo-caption"><span><strong>实际应用输入流程</strong><small>真实软件录屏 · 中文输入、翻译与上屏结果</small><small>建议全屏观看，手机可横屏查看</small></span></figcaption>
            </figure>
          </div>
          <p className="home-preview-note" data-motion="enter" data-motion-delay="360">视频为真实软件录屏；Hero 仍为预设文本演示。截图保留原始软件界面，设置关于页版本为 0.1.0；候选译文就绪展示翻译结果。<a href="https://github.com/boluomi2001/ctrllingo#界面预览" target="_blank" rel="noreferrer">查看项目仓库界面预览</a></p>
        </div>
      </section>

      <section className="home-section" id="story" aria-labelledby="home-story-title">
        <div className="home-container home-story-layout">
          <article className="home-story-copy" data-motion="enter">
            <p className="home-eyebrow">DESIGN STORY</p>
            <h2 id="home-story-title">为什么我要做 CtrlLingo？</h2>
            <p>在日常使用输入法时，我希望把中文组织、翻译和外语输出接在同一个流程里，减少反复复制切换。我基于青简开源输入法，围绕 Ctrl 按需翻译、Space 上屏、多语言切换和费用信息展示设计交互规则，并借助 AI 辅助编码推进迭代。</p>
            <p>0.1.0 已有本人试用反馈，0.1.1 的原生安装与更多应用兼容性、真实账户对照仍待补充。</p>
            <p className="home-story-boundary">底层输入引擎与模型沿用上游，不将全部代码、模型和已有测试归为个人原创。</p>
          </article>
          <aside className="home-card home-contributions" aria-label="我的主要贡献" data-motion="enter" data-motion-delay="100" data-pointer="card">
            <h3>我的主要贡献</h3>
            <ol>
              <li>定义多语言输入的产品方向与核心需求。</li>
              <li>设计 Ctrl / Space 流程及等待、失败、编辑后的反馈规则。</li>
              <li>规划用量、费用估算和账户余额的展示边界。</li>
              <li>拆解开发任务，借助 AI 辅助编码推进实现。</li>
              <li>根据本人试用反馈检查与调整体验。</li>
            </ol>
          </aside>
        </div>
      </section>

      <section className="home-section" id="open-source" aria-labelledby="home-open-source-title">
        <div className="home-container home-open-source-panel" data-motion="enter" data-pointer="card">
          <div className="home-open-source-copy">
            <p className="home-eyebrow">OPEN SOURCE</p>
            <h2 id="home-open-source-title">基于开源，持续迭代。</h2>
            <p>CtrlLingo 基于青简（Qingjian）二次开发，聚焦多语言输入交互与用量信息展示。</p>
            <p>独立项目，由 boluomi2001 维护，非青简官方产品。</p>
            <small>代码采用 GPL-3.0-or-later；随包数据与部分组件保留各自来源和许可。</small>
          </div>
          <nav className="home-source-links" aria-label="项目与许可链接">
            <a href="https://github.com/boluomi2001/ctrllingo" target="_blank" rel="noreferrer">项目仓库 <ArrowUpRight size={16} aria-hidden="true" /></a>
            <a href="https://github.com/qingjian-team/qingjian" target="_blank" rel="noreferrer">青简上游 <ArrowUpRight size={16} aria-hidden="true" /></a>
            <a href="https://github.com/boluomi2001/ctrllingo/blob/main/docs/UPSTREAM.md" target="_blank" rel="noreferrer">许可与归属 <ArrowUpRight size={16} aria-hidden="true" /></a>
            <a href="https://github.com/boluomi2001/ctrllingo/blob/main/LICENSE" target="_blank" rel="noreferrer">GPL-3.0-or-later <ArrowUpRight size={16} aria-hidden="true" /></a>
          </nav>
        </div>
      </section>

      <section className="home-section" id="faq" aria-labelledby="home-faq-title">
        <div className="home-container home-faq-container">
          <header className="home-section-heading" data-motion="enter">
            <p className="home-eyebrow">FAQ</p>
            <h2 id="home-faq-title">常见问题</h2>
          </header>
          <div className="home-faq-list">
            {faq.map((item, index) => (
              <details className="home-faq-item" key={item.question} open={index === 0 ? true : undefined} data-motion="enter" data-motion-delay={index * 70} data-pointer="card">
                <summary><span>{item.question}</span><ChevronDown size={20} aria-hidden="true" /></summary>
                <p>{item.answer}</p>
              </details>
            ))}
          </div>
        </div>
      </section>

      <dialog className="home-image-dialog" ref={dialogRef} aria-label="产品界面大图" onClose={() => openerRef.current?.focus()}>
        {image && <div className="home-image-dialog-content"><button className="home-image-dialog-close" type="button" onClick={() => dialogRef.current?.close()}>关闭</button><img src={image.src} alt={image.alt} width={image.width} height={image.height} decoding="async" /><p>{image.title}：{image.caption}</p></div>}
      </dialog>
    </div>
  )
}

export default HomeSections
