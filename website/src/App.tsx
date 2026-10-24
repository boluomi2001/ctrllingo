import { useEffect, useRef, useState, type MouseEvent } from 'react'
import { Download, ArrowRight, Menu, X } from 'lucide-react'
import { siGithub } from 'simple-icons'
import HeroDemo from './HeroDemo'
import HomeSections from './HomeSections'
import { FooterCTA, Footer } from './HomeFooter'
import { usePageMotion } from './usePageMotion'
import { usePointerEffects } from './usePointerEffects'
import './page-motion.css'
import './pointer-effects.css'
import './landing-effects.css'

const repository = 'https://github.com/boluomi2001/ctrllingo'
const installerUrl = 'https://github.com/boluomi2001/ctrllingo/releases/download/v0.1.1/CtrlLingo-0.1.1-windows-x64-setup.exe'

function GithubMark() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" className="github-mark">
      <path d={siGithub.path} />
    </svg>
  )
}

function App() {
  const [menuOpen, setMenuOpen] = useState(false)
  const pageRef = useRef<HTMLDivElement>(null)
  const menuRef = useRef<HTMLDivElement>(null)
  const menuToggleRef = useRef<HTMLButtonElement>(null)
  usePageMotion(pageRef)
  usePointerEffects(pageRef)

  useEffect(() => {
    if (!menuOpen) return
    menuRef.current?.querySelector<HTMLAnchorElement>('a')?.focus()
  }, [menuOpen])

  const focusDemo = (event: MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault()
    setMenuOpen(false)
    const demo = document.getElementById('demo')
    demo?.focus({ preventScroll: true })
    demo?.scrollIntoView({ behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth', block: 'center' })
  }

  const goToSection = (event: MouseEvent<HTMLAnchorElement>) => {
    event.preventDefault()
    setMenuOpen(false)
    const hash = event.currentTarget.hash
    const section = document.getElementById(hash.slice(1))
    const heading = section?.querySelector<HTMLElement>('h2')
    heading?.setAttribute('tabindex', '-1')
    heading?.classList.add('section-anchor')
    heading?.focus({ preventScroll: true })
    section?.scrollIntoView({ behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth', block: 'start' })
    window.history.replaceState(null, '', hash)
  }

  const navLinks = (
    <>
      <a className="nav-motion-link" href="#features" onClick={goToSection}>产品功能</a>
      <a className="nav-motion-link" href="#how-it-works" onClick={goToSection}>使用方式</a>
      <a className="nav-motion-link" href="#story" onClick={goToSection}>设计故事</a>
      <a className="nav-motion-link" href="#faq" onClick={goToSection}>常见问题</a>
      <a className="nav-motion-link" href="#open-source" onClick={goToSection}>开源说明</a>
    </>
  )

  return (
    <div className="site-shell" id="top" ref={pageRef}>
      <a className="skip-link" href="#main-content">跳到主要内容</a>
      <header className="site-header">
        <a className="brand" href="#top" aria-label="CtrlLingo 译点，回到页首">
          <img src="/media/ctrldot-logo-96.webp" alt="" width="96" height="96" decoding="async" />
          <span className="brand-name">CtrlLingo <span>译点</span></span>
        </a>

        <nav className="desktop-nav" aria-label="主导航">{navLinks}</nav>

        <div className="header-actions">
          <a className="button button-primary header-cta" data-pointer="magnet" href={installerUrl} title="下载 CtrlLingo 0.1.1 Windows x64 安装包">
            <Download size={16} aria-hidden="true" /> 立即下载
          </a>
          <a className="button button-outline header-github" data-pointer="magnet" href={repository} target="_blank" rel="noreferrer">
            <GithubMark /> GitHub
          </a>
          <button
            className="menu-toggle"
            ref={menuToggleRef}
            aria-label={menuOpen ? '关闭菜单' : '打开菜单'}
            aria-expanded={menuOpen}
            aria-controls="mobile-navigation"
            onClick={() => setMenuOpen((open) => !open)}
          >
            {menuOpen ? <X size={22} /> : <Menu size={22} />}
          </button>
        </div>

          <div
            id="mobile-navigation"
            className="mobile-nav-panel"
            ref={menuRef}
            hidden={!menuOpen}
            inert={!menuOpen}
            onKeyDown={(event) => {
              if (event.key === 'Escape') {
                setMenuOpen(false)
                menuToggleRef.current?.focus()
              }
            }}
          >
            <nav aria-label="移动端主导航">{navLinks}</nav>
            <a className="button button-primary" data-pointer="magnet" href={installerUrl} title="下载 CtrlLingo 0.1.1 Windows x64 安装包" onClick={() => { setMenuOpen(false); menuToggleRef.current?.focus() }}>
              <Download size={16} aria-hidden="true" /> 立即下载
            </a>
            <a className="button button-outline" data-pointer="magnet" href={repository} target="_blank" rel="noreferrer" onClick={() => setMenuOpen(false)}>
              <GithubMark /> GitHub
            </a>
          </div>
      </header>

      <main id="main-content" tabIndex={-1}>
        <section className="hero" id="intro" aria-labelledby="hero-title">
          <div className="hero-copy">
            <p className="eyebrow" data-motion="enter"><span aria-hidden="true" /> AI MULTILINGUAL INPUT ASSISTANT</p>
            <h1 id="hero-title"><span className="hero-title-line hero-title-first" data-motion="enter" data-motion-delay="60">输入中文，</span><br /><span className="hero-title-line hero-title-world" data-pointer="gradient" data-motion="enter" data-motion-delay="150">表达世界。</span></h1>
            <p className="hero-subtitle" data-motion="enter" data-motion-delay="200">让语言切换，像输入一样自然。</p>
            <p className="hero-description" data-motion="enter" data-motion-delay="240">在当前输入位置，用中文组织句子，按需翻译为目标语言，再将译文直接上屏。</p>

            <div className="hero-actions" data-motion="enter" data-motion-delay="280">
              <a className="button button-primary" data-pointer="magnet" href={installerUrl} title="下载 CtrlLingo 0.1.1 Windows x64 安装包">
                <Download size={17} aria-hidden="true" /> 立即下载
              </a>
              <a className="button button-outline" data-pointer="magnet" href={repository} target="_blank" rel="noreferrer">
                <GithubMark /> GitHub <ArrowRight size={15} aria-hidden="true" />
              </a>
            </div>

            <ul className="hero-points" data-motion="enter" data-motion-delay="320" aria-label="主要特性">
              <li><span className="point-key">Ctrl</span> 按需翻译</li>
              <li><span className="point-key point-space">Space</span> 译文上屏</li>
              <li><span className="point-language" aria-hidden="true">中</span> 五种目标语言</li>
            </ul>
            <p className="release-note" data-motion="enter" data-motion-delay="360">Windows 二次开发测试版 · 首次使用需配置翻译服务</p>
          </div>

          <HeroDemo />
        </section>
        <HomeSections />
        <FooterCTA onDemo={focusDemo} />
      </main>

      <Footer />
    </div>
  )
}

export default App
