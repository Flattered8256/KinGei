export function usePresence() {
  const store = usePresenceStore()
  const sseStatus = useSseStatus()
  const { user } = useAuth()

  function isOnline(id?: string | null) {
    return store.isOnline(id)
  }

  function markSelf() {
    if (sseStatus.value === 'connected' && user.value) {
      store.set(user.value.id, true)
    }
  }

  async function refresh() {
    await store.refresh()
    markSelf()
  }

  return {
    isOnline,
    markSelf,
    ingest: store.ingest,
    refresh,
    reset: store.reset,
  }
}
