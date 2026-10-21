const QUERY = '(max-width: 899px)'

export function useIsMobile() {
  const isMobile = useState('ui.isMobile', () => false)

  function sync() {
    if (!import.meta.client) {
      return
    }
    isMobile.value = window.matchMedia(QUERY).matches
  }

  function init() {
    if (!import.meta.client) {
      return
    }
    sync()
    window.matchMedia(QUERY).addEventListener('change', sync)
  }

  return {
    isMobile,
    init,
  }
}
