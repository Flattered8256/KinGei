export type ThemePreference = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'kingei.theme'

export function useTheme() {
  const preference = useState<ThemePreference>('theme.preference', () => 'system')
  const resolved = useState<'light' | 'dark'>('theme.resolved', () => 'light')

  function readStored(): ThemePreference {
    if (!import.meta.client) {
      return 'system'
    }
    const value = localStorage.getItem(STORAGE_KEY)
    if (value === 'light' || value === 'dark' || value === 'system') {
      return value
    }
    return 'system'
  }

  function systemTheme(): 'light' | 'dark' {
    if (!import.meta.client) {
      return 'light'
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  function apply(next = preference.value) {
    const theme = next === 'system' ? systemTheme() : next
    resolved.value = theme
    if (!import.meta.client) {
      return
    }
    document.documentElement.dataset.theme = theme
  }

  function setPreference(next: ThemePreference) {
    preference.value = next
    if (import.meta.client) {
      localStorage.setItem(STORAGE_KEY, next)
    }
    apply(next)
  }

  function init() {
    preference.value = readStored()
    apply(preference.value)
    if (!import.meta.client) {
      return
    }
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (preference.value === 'system') {
        apply('system')
      }
    })
  }

  return {
    preference,
    resolved,
    setPreference,
    apply,
    init,
  }
}
