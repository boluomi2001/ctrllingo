import { useEffect, type RefObject } from 'react'

export function usePageMotion(scope: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const root = scope.current
    if (!root) return
    const preference = window.matchMedia('(prefers-reduced-motion: reduce)')
    const active = new Set<Animation>()
    const play = (element: HTMLElement, frames: Keyframe[], duration: number, delay = 0, easing = 'cubic-bezier(.22, 1, .36, 1)') => {
      const animation = element.animate(frames, { duration, delay, easing, fill: 'backwards' })
      animation.id = `page-${element.dataset.motion || 'step-badge'}`
      active.add(animation)
      animation.onfinish = animation.oncancel = () => active.delete(animation)
    }
    const observer = new IntersectionObserver(entries => {
      for (const entry of entries) {
        if (!entry.isIntersecting || entry.intersectionRatio < .12) continue
        const element = entry.target as HTMLElement
        observer.unobserve(element)
        if (preference.matches || document.hidden) continue
        const delay = Number(element.dataset.motionDelay || 0)
        if (element.dataset.motion === 'float') {
          play(element, [{ translate: '0 0' }, { translate: '0 -8px', offset: .4 }, { translate: '0 0' }], 2800)
        } else if (element.dataset.motion === 'step') {
          play(element, [{ opacity: 0, translate: '0 28px' }, { opacity: 1, translate: '0 0' }], 700, delay)
          const stepDelay = delay * 3
          const shadow = getComputedStyle(element).boxShadow
          play(element, [{ boxShadow: shadow }, { boxShadow: '0 0 0 2px rgb(143 175 157 / 45%), 0 12px 26px rgb(65 109 86 / 10%)', offset: .35 }, { boxShadow: shadow }], 1200, stepDelay)
          const badge = element.querySelector<HTMLElement>('.home-step-number')
          if (badge) play(badge, [{ scale: '1' }, { scale: '1.1', offset: .35 }, { scale: '1' }], 1000, stepDelay)
          const connector = element.querySelector<HTMLElement>('.home-step-link-progress')
          if (connector) play(connector, [{ scale: window.innerWidth < 640 ? '1 0' : '0 1', opacity: .2 }, { scale: '1 1', opacity: 1 }], 1100, stepDelay + 280, 'linear')
        } else {
          play(element, [{ opacity: 0, translate: '0 28px' }, { opacity: 1, translate: '0 0' }], 700, delay)
        }
      }
    }, { threshold: .12 })
    root.querySelectorAll<HTMLElement>('[data-motion]').forEach(element => observer.observe(element))
    const cancel = () => active.forEach(animation => animation.cancel())
    const onPreference = () => { if (preference.matches) cancel() }
    const onVisibility = () => { if (document.hidden) cancel() }
    preference.addEventListener('change', onPreference)
    document.addEventListener('visibilitychange', onVisibility)
    return () => {
      observer.disconnect()
      cancel()
      preference.removeEventListener('change', onPreference)
      document.removeEventListener('visibilitychange', onVisibility)
    }
  }, [scope])
}
