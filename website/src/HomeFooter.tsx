import { ArrowRight, Play } from 'lucide-react'
import { siGithub } from 'simple-icons'
import type { MouseEvent } from 'react'
import './home-footer.css'

function GithubMark() {
  return <svg aria-hidden="true" viewBox="0 0 24 24" className="github-mark"><path d={siGithub.path} /></svg>
}

export function FooterCTA({ onDemo }: { onDemo: (event: MouseEvent<HTMLAnchorElement>) => void }) {
  return (
    <section className="home-footer-cta" id="get-started" aria-labelledby="footer-cta-title">
      <div className="home-container home-cta-panel" data-motion="enter" data-pointer="surface">
        <img className="home-cta-hills" src="/media/footer-hills.webp" alt="" aria-hidden="true" width="2172" height="724" decoding="async" loading="lazy" />
        <div className="home-cta-copy">
          <p className="home-cta-eyebrow">LET YOUR WORDS GO FURTHER</p>
          <h2 id="footer-cta-title" className="pointer-title" data-pointer="gradient">少一点切换，多一点表达。</h2>
          <p>体验 CtrlLingo · 译点，让翻译接入你的输入流程。</p>
          <div className="home-cta-actions">
            <a className="button button-primary" href="#demo" onClick={onDemo}><Play size={17} aria-hidden="true" /> 查看产品演示</a>
            <a className="button button-outline" data-pointer="magnet" href="https://github.com/boluomi2001/ctrllingo" target="_blank" rel="noreferrer"><GithubMark /> GitHub <ArrowRight size={15} aria-hidden="true" /></a>
          </div>
          <a className="home-release-link" href="https://github.com/boluomi2001/ctrllingo/releases/tag/v0.1.1" target="_blank" rel="noreferrer">获取 Windows x64 版本与使用说明 <ArrowRight size={15} aria-hidden="true" /></a>
        </div>
        <img className="home-cta-logo" data-motion="float" src="/media/ctrldot-logo-512.webp" alt="" aria-hidden="true" width="512" height="512" decoding="async" loading="lazy" />
      </div>
    </section>
  )
}

export function Footer() {
  return (
    <footer className="page-footer">
      <div className="home-container">
        <div className="page-footer-main" data-motion="enter">
          <div>
            <a className="brand" href="#top" aria-label="CtrlLingo 译点，回到页首">
              <img src="/media/ctrldot-logo-96.webp" alt="" width="96" height="96" decoding="async" loading="lazy" />
              <span className="brand-name">CtrlLingo <span>译点</span></span>
            </a>
            <p className="page-footer-tagline">AI 多语言输入助手</p>
          </div>
          <nav className="page-footer-links" aria-label="页脚导航">
            <a href="#features">产品功能</a>
            <a href="#story">设计故事</a>
            <a href="#open-source">开源说明</a>
            <a href="https://github.com/boluomi2001/ctrllingo" target="_blank" rel="noreferrer">GitHub</a>
          </nav>
        </div>
        <div className="page-footer-credits" data-motion="enter" data-motion-delay="100">
          <p>基于 <a href="https://github.com/qingjian-team/qingjian" target="_blank" rel="noreferrer">Qingjian 青简</a> 开源项目二次开发</p>
          <p>© {new Date().getFullYear()} CtrlLingo · <a href="https://github.com/boluomi2001/ctrllingo/blob/main/LICENSE" target="_blank" rel="noreferrer">输入法 GPL-3.0-or-later</a></p>
        </div>
      </div>
    </footer>
  )
}
