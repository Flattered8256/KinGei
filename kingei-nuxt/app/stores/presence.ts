import type { ClientEvent } from '~/types/api'
import { getPresence } from '~/utils/api/presence'

export const usePresenceStore = defineStore('presence', () => {
  const online = ref<Record<string, true>>({})

  function isOnline(id?: string | null) {
    return Boolean(id && online.value[id])
  }

  function replace(ids: string[]) {
    const next: Record<string, true> = {}
    for (const id of ids) {
      next[id] = true
    }
    online.value = next
  }

  function set(id: string, value: boolean) {
    if (value) {
      if (online.value[id]) {
        return
      }
      online.value = { ...online.value, [id]: true }
      return
    }
    if (!online.value[id]) {
      return
    }
    const { [id]: _, ...rest } = online.value
    online.value = rest
  }

  function ingest(event: ClientEvent) {
    if (event.event !== 'presence.changed') {
      return
    }
    set(event.data.user_id, event.data.online)
  }

  async function refresh() {
    const entries = await getPresence()
    replace(entries.filter(entry => entry.online).map(entry => entry.user_id))
  }

  function reset() {
    online.value = {}
  }

  return {
    online,
    isOnline,
    replace,
    set,
    ingest,
    refresh,
    reset,
  }
})
