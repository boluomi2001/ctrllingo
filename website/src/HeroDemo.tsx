import { useEffect, useRef, useState, type KeyboardEvent } from 'react'
import { ArrowDown, Check, Pause, Play, RotateCcw } from 'lucide-react'
import './hero-motion.css'

type DemoState = 'ready' | 'translating' | 'translated' | 'committed'

const translation = 'I want to watch a movie after playing table tennis.'

function HeroDemo() {
  const [state, setState] = useState<DemoState>('ready')
  const [motionVisible, setMotionVisible] = useState(false)
  const [reducedMotion, setReducedMotion] = useState(false)
  const [motionPaused, setMotionPaused] = useState(false)
  const rootRef = useRef<HTMLElement>(null)
  const pointerFrameRef = useRef<number | undefined>(undefined)
  const pointerRef = useRef({ x: 0, y: 0 })
  const timerRef = useRef<number | undefined>(undefined)
  const soloCtrlRef = useRef(false)
  const motionPausedRef = useRef(false)
  const syncMotionRef = useRef<() => void>(() => {})

  useEffect(() => {
    const stage = rootRef.current
    if (!stage) return

    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    const pointerQuery = window.matchMedia('(min-width: 1024px) and (hover: hover) and (pointer: fine)')
    const artLayers = Array.from(stage.querySelectorAll<HTMLElement>('.hero-art-layer'))
    setReducedMotion(motionQuery.matches)
    let isIntersecting = false
    const observer = new IntersectionObserver((entries) => {
      const entry = entries[entries.length - 1]
      isIntersecting = Boolean(entry?.isIntersecting && entry.intersectionRatio >= 0.25)
      setMotionVisible(isIntersecting && document.visibilityState === 'visible')
      syncMotion()
    }, { threshold: [0, 0.25] })
    observer?.observe(stage)

    const clearPointer = () => {
      if (pointerFrameRef.current !== undefined) window.cancelAnimationFrame(pointerFrameRef.current)
      pointerFrameRef.current = undefined
      artLayers.forEach((layer) => {
        layer.style.setProperty('--demo-pointer-x', '0px')
        layer.style.setProperty('--demo-pointer-y', '0px')
      })
    }
    const setPointer = (event: PointerEvent) => {
      pointerRef.current = { x: event.clientX, y: event.clientY }
      if (pointerFrameRef.current !== undefined) return
      pointerFrameRef.current = window.requestAnimationFrame(() => {
        pointerFrameRef.current = undefined
        const rect = stage.getBoundingClientRect()
        const x = Math.max(-1, Math.min(1, ((pointerRef.current.x - rect.left) / rect.width - 0.5) * 2))
        const y = Math.max(-1, Math.min(1, ((pointerRef.current.y - rect.top) / rect.height - 0.5) * 2))
        artLayers.forEach((layer) => {
          const distance = layer.classList.contains('ctrl-layer') ? 16 : layer.classList.contains('space-layer') ? -20 : -10
          layer.style.setProperty('--demo-pointer-x', `${x * distance}px`)
          layer.style.setProperty('--demo-pointer-y', `${y * distance}px`)
        })
      })
    }
    let pointerEnabled = false
    const syncMotion = () => {
      const active = pointerQuery.matches && !motionQuery.matches && isIntersecting && document.visibilityState === 'visible' && !motionPausedRef.current
      if (active && !pointerEnabled) {
        stage.addEventListener('pointermove', setPointer)
        stage.addEventListener('pointerleave', clearPointer)
        pointerEnabled = true
      } else if (!active && pointerEnabled) {
        stage.removeEventListener('pointermove', setPointer)
        stage.removeEventListener('pointerleave', clearPointer)
        pointerEnabled = false
        clearPointer()
      }
    }
    syncMotionRef.current = syncMotion
    const onMotionPreferenceChange = () => {
      setReducedMotion(motionQuery.matches)
      syncMotion()
    }
    const onVisibilityChange = () => {
      setMotionVisible(isIntersecting && document.visibilityState === 'visible')
      syncMotion()
    }
    const onPointerCapabilityChange = () => syncMotion()
    pointerQuery.addEventListener('change', onPointerCapabilityChange)
    motionQuery.addEventListener('change', onMotionPreferenceChange)
    document.addEventListener('visibilitychange', onVisibilityChange)
    syncMotion()

    return () => {
      window.clearTimeout(timerRef.current)
      observer?.disconnect()
      pointerQuery.removeEventListener('change', onPointerCapabilityChange)
      motionQuery.removeEventListener('change', onMotionPreferenceChange)
      document.removeEventListener('visibilitychange', onVisibilityChange)
      stage.removeEventListener('pointermove', setPointer)
      stage.removeEventListener('pointerleave', clearPointer)
      clearPointer()
      syncMotionRef.current = () => {}
    }
  }, [])

  const translate = () => {
    if (state !== 'ready') return
    setState('translating')
    timerRef.current = window.setTimeout(() => setState('translated'), 600)
  }

  const commit = () => {
    if (state === 'translated') setState('committed')
  }

  const reset = () => {
    window.clearTimeout(timerRef.current)
    soloCtrlRef.current = false
    setState('ready')
    rootRef.current?.focus()
  }

  const toggleMotion = () => {
    const paused = !motionPausedRef.current
    motionPausedRef.current = paused
    setMotionPaused(paused)
    syncMotionRef.current()
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget || event.repeat) return
    if (event.key === 'Control') {
      if (state === 'ready' && !event.altKey && !event.metaKey && !event.shiftKey) soloCtrlRef.current = true
      return
    }
    if (event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) soloCtrlRef.current = false
    if (event.key === ' ' && !event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
      event.preventDefault()
      commit()
    }
  }

  const handleKeyUp = (event: KeyboardEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget || event.key !== 'Control') return
    if (soloCtrlRef.current && !event.altKey && !event.metaKey && !event.shiftKey) translate()
    soloCtrlRef.current = false
  }

  const clearSoloCtrl = () => { soloCtrlRef.current = false }
  const isTranslated = state === 'translated' || state === 'committed'
  const motionActive = motionVisible && !motionPaused && !reducedMotion
  const liveMessage = state === 'ready'
    ? '交互演示，预设中文文本。聚焦此区域后可单独按下并松开 Ctrl 进行演示。'
    : state === 'translating'
      ? '正在演示翻译，请稍候。'
      : state === 'translated'
        ? `演示译文已就绪：${translation}。按 Space 或点击确认上屏。`
        : '演示译文已在目标输入区域上屏。'

  return (
    <section
      className="demo-stage"
      data-demo-state={state}
      data-motion-active={motionActive}
      id="demo"
      aria-label="多语言输入交互演示"
      aria-describedby="demo-disclaimer"
      tabIndex={0}
      ref={rootRef}
      onKeyDown={handleKeyDown}
      onKeyUp={handleKeyUp}
      onBlur={clearSoloCtrl}
    >
      <div className="hero-art-layer glass-layer" aria-hidden="true">
        <div className="hero-art-float glass-float"><img className="glass-ribbon" src="/media/hero-glass.webp" alt="" width="1672" height="941" decoding="async" /></div>
      </div>
      <div className="demo-window" data-motion="enter" data-motion-delay="200">
        <div className="window-topbar">
          <span className="window-dot dot-coral" aria-hidden="true" /><span className="window-dot dot-amber" aria-hidden="true" /><span className="window-dot dot-green" aria-hidden="true" />
          <span className="window-caption">预设文本演示</span>
          <button className="window-reset" onClick={reset} disabled={state === 'ready'} aria-label="重新演示" title="重新演示">
            <RotateCcw size={14} aria-hidden="true" />
          </button>
          <span className="language-pair">中文 <span>→</span> English</span>
        </div>

        <div className="demo-content">
          <p className="field-label">输入内容</p>
          <div className="input-card">我想打完乒乓球后去看电影</div>

          <div className="demo-control-row">
            <button className="key-button ctrl-button" onClick={translate} disabled={state !== 'ready'}>
              <span className="key-cap">Ctrl</span><span>{state === 'translating' ? '翻译中…' : '翻译整句'}</span>
            </button>
          </div>
          <ArrowDown className="demo-arrow" size={22} aria-hidden="true" />

          <p className="field-label result-label">{state === 'committed' ? '演示输入区域 · 已上屏' : '译文预览'}</p>
          <div className={`translation-card ${isTranslated ? 'translation-ready' : ''} ${state === 'committed' ? 'is-committed' : ''}`} aria-label={state === 'committed' ? '演示目标输入区域，译文已上屏' : '英文译文预览'}>
            {state === 'ready' && <span className="waiting-copy">按下 Ctrl，预览整句译文</span>}
            {state === 'translating' && <span className="loading-copy"><i /> 正在准备译文</span>}
            {isTranslated && <span lang="en">{translation}</span>}
            {state === 'committed' && <Check className="translation-check" size={18} aria-label="已上屏" />}
          </div>

          <div className="demo-control-row space-row">
            {state === 'committed' ? (
              <button className="key-button reset-button" onClick={reset}>
                <RotateCcw size={15} aria-hidden="true" /> 重新演示
              </button>
            ) : (
              <button className="key-button space-button" onClick={commit} disabled={state !== 'translated'}>
                <span className="key-cap space-cap">Space</span><span>确认上屏</span>
              </button>
            )}
          </div>

        </div>

        <div className="demo-footnote" id="demo-disclaimer">
          <span>交互演示 · 预设文本</span><span>不会操作真实输入法</span>
        </div>
        <span className="status-sr-only" aria-live="polite">{liveMessage}</span>
      </div>

      <div className="hero-art-layer ctrl-layer" aria-hidden="true">
        <div className="hero-art-float ctrl-float"><img className="key-art ctrl-art" src="/media/ctrl-key-512.webp" alt="" width="512" height="512" decoding="async" /></div>
      </div>
      <div className="hero-art-layer space-layer" aria-hidden="true">
        <div className="hero-art-float space-float"><img className="key-art space-art" src="/media/space-key-512.webp" alt="" width="512" height="512" decoding="async" /></div>
      </div>
      <button
        className="motion-toggle"
        type="button"
        aria-label={reducedMotion ? '静态模式，动效跟随系统设置' : motionPaused ? '播放装饰动效' : '暂停装饰动效'}
        title={reducedMotion ? '动效跟随系统减少动态效果设置' : motionPaused ? '播放装饰动效' : '暂停装饰动效'}
        disabled={reducedMotion}
        onClick={toggleMotion}
      >
        {reducedMotion ? null : motionPaused ? <Play size={14} aria-hidden="true" /> : <Pause size={14} aria-hidden="true" />}
        {reducedMotion ? '静态模式' : motionPaused ? '播放动效' : '暂停动效'}
      </button>
    </section>
  )
}

export default HeroDemo
