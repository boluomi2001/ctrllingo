import { useEffect, type RefObject } from 'react'

type PointerPosition = {
  x: number
  y: number
  bounds: DOMRect
}

const POINTER_QUERY = '(min-width: 1024px) and (hover: hover) and (pointer: fine) and (prefers-reduced-motion: no-preference)'
const POINTER_ELEMENTS = '[data-pointer~="card"], [data-pointer~="magnet"], [data-pointer~="gradient"], [data-pointer~="surface"]'

export function usePointerEffects(scope: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const root = scope.current
    if (!root) return

    const media = window.matchMedia(POINTER_QUERY)
    const elements = [...root.querySelectorAll<HTMLElement>(POINTER_ELEMENTS)]
    const bounds = new Map<HTMLElement, DOMRect>()
    const pending = new Map<HTMLElement, PointerPosition>()
    let frame = 0

    const cancelFrame = () => {
      if (!frame) return
      cancelAnimationFrame(frame)
      frame = 0
    }

    const clearElement = (element: HTMLElement) => {
      bounds.delete(element)
      pending.delete(element)
      element.removeAttribute('data-pointer-active')
      element.style.removeProperty('--pointer-x')
      element.style.removeProperty('--pointer-y')
      element.style.removeProperty('--pointer-dx')
      element.style.removeProperty('--pointer-dy')
      element.style.removeProperty('--tilt-x')
      element.style.removeProperty('--tilt-y')
      element.style.removeProperty('--magnet-x')
      element.style.removeProperty('--magnet-y')
      if (!pending.size) cancelFrame()
    }

    const clearAll = () => {
      cancelFrame()
      pending.clear()
      bounds.clear()
      elements.forEach(element => {
        element.removeAttribute('data-pointer-active')
        element.style.removeProperty('--pointer-x')
        element.style.removeProperty('--pointer-y')
        element.style.removeProperty('--pointer-dx')
        element.style.removeProperty('--pointer-dy')
        element.style.removeProperty('--tilt-x')
        element.style.removeProperty('--tilt-y')
        element.style.removeProperty('--magnet-x')
        element.style.removeProperty('--magnet-y')
      })
    }

    const flush = () => {
      frame = 0
      for (const [element, position] of pending) {
        const { x, y, bounds: rect } = position
        const dx = Math.max(-1, Math.min(1, ((x - rect.left) / rect.width) * 2 - 1))
        const dy = Math.max(-1, Math.min(1, ((y - rect.top) / rect.height) * 2 - 1))
        element.style.setProperty('--pointer-x', `${x - rect.left}px`)
        element.style.setProperty('--pointer-y', `${y - rect.top}px`)
        element.style.setProperty('--pointer-dx', `${dx}`)
        element.style.setProperty('--pointer-dy', `${dy}`)
        if (element.matches('[data-pointer~="card"]')) {
          element.style.setProperty('--tilt-x', `${-dy * 3}deg`)
          element.style.setProperty('--tilt-y', `${dx * 3}deg`)
        }
        if (element.matches('[data-pointer~="magnet"]')) {
          const limit = element.closest('.header-actions') ? 5 : 8
          element.style.setProperty('--magnet-x', `${dx * limit}px`)
          element.style.setProperty('--magnet-y', `${dy * limit}px`)
        }
      }
      pending.clear()
    }

    const schedule = (element: HTMLElement, event: PointerEvent, rect: DOMRect) => {
      if (event.buttons !== 0) return
      pending.set(element, { x: event.clientX, y: event.clientY, bounds: rect })
      if (!frame) frame = requestAnimationFrame(flush)
    }

    const onEnter = (event: PointerEvent) => {
      const element = event.currentTarget as HTMLElement
      const rect = element.getBoundingClientRect()
      bounds.set(element, rect)
      element.dataset.pointerActive = 'true'
      schedule(element, event, rect)
    }

    const onMove = (event: PointerEvent) => {
      const element = event.currentTarget as HTMLElement
      if (event.buttons !== 0) return
      const rect = bounds.get(element) || element.getBoundingClientRect()
      bounds.set(element, rect)
      element.dataset.pointerActive = 'true'
      schedule(element, event, rect)
    }

    const onLeave = (event: PointerEvent) => clearElement(event.currentTarget as HTMLElement)
    const onBlur = (event: FocusEvent) => clearElement(event.currentTarget as HTMLElement)
    const onDown = (event: PointerEvent) => {
      const element = event.currentTarget as HTMLElement
      pending.delete(element)
      bounds.delete(element)
      if (!pending.size) cancelFrame()
    }

    const sync = () => {
      clearAll()
      if (!media.matches) return
      for (const element of elements) {
        element.addEventListener('pointerenter', onEnter)
        element.addEventListener('pointermove', onMove)
        element.addEventListener('pointerleave', onLeave)
        element.addEventListener('pointerdown', onDown)
        element.addEventListener('blur', onBlur)
      }
      window.addEventListener('scroll', clearAll, true)
      window.addEventListener('resize', clearAll)
      window.addEventListener('blur', clearAll)
      document.addEventListener('visibilitychange', onVisibilityChange)
    }

    const unbind = () => {
      for (const element of elements) {
        element.removeEventListener('pointerenter', onEnter)
        element.removeEventListener('pointermove', onMove)
        element.removeEventListener('pointerleave', onLeave)
        element.removeEventListener('pointerdown', onDown)
        element.removeEventListener('blur', onBlur)
      }
      window.removeEventListener('scroll', clearAll, true)
      window.removeEventListener('resize', clearAll)
      window.removeEventListener('blur', clearAll)
      document.removeEventListener('visibilitychange', onVisibilityChange)
      clearAll()
    }

    function onVisibilityChange() {
      if (document.hidden) clearAll()
    }

    const onMediaChange = () => {
      unbind()
      sync()
    }

    media.addEventListener('change', onMediaChange)
    sync()
    return () => {
      media.removeEventListener('change', onMediaChange)
      unbind()
    }
  }, [scope])
}
