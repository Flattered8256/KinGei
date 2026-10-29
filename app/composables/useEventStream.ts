import type { ClientEvent } from '~/types/api'
import {
  connectEventStream,
  type EventStreamConnection,
} from '~/utils/api/event'

export interface UseEventStreamOptions {
  onEvent: (event: ClientEvent) => void
  onReconnect?: () => void
  immediate?: boolean
}

export function useEventStream(options: UseEventStreamOptions) {
  const { isAuthenticated } = useAuth()
  const connected = ref(false)
  let connection: EventStreamConnection | null = null

  function start() {
    if (!import.meta.client || connection || !isAuthenticated.value) {
      return
    }
    const conn = connectEventStream({
      onEvent: options.onEvent,
      onReconnect: options.onReconnect,
      onOpen: () => {
        if (connection === conn) {
          connected.value = true
        }
      },
      onClose: () => {
        if (connection === conn) {
          connection = null
          connected.value = false
        }
      },
    })
    connection = conn
  }

  function stop() {
    const conn = connection
    connection = null
    connected.value = false
    conn?.stop()
  }

  function wake() {
    connection?.wake()
  }

  if (import.meta.client) {
    watch(
      isAuthenticated,
      (authenticated) => {
        if (!authenticated) {
          stop()
          return
        }
        if (options.immediate !== false) {
          start()
        }
      },
      { immediate: true },
    )

    const onPageHide = () => {
      stop()
    }
    const onPageShow = () => {
      if (!isAuthenticated.value || options.immediate === false) {
        return
      }
      start()
      options.onReconnect?.()
    }
    const onVisibility = () => {
      if (document.visibilityState === 'visible') {
        wake()
      }
    }
    const onOnline = () => {
      wake()
    }

    window.addEventListener('pagehide', onPageHide)
    window.addEventListener('pageshow', onPageShow)
    document.addEventListener('visibilitychange', onVisibility)
    window.addEventListener('online', onOnline)

    onBeforeUnmount(() => {
      window.removeEventListener('pagehide', onPageHide)
      window.removeEventListener('pageshow', onPageShow)
      document.removeEventListener('visibilitychange', onVisibility)
      window.removeEventListener('online', onOnline)
      stop()
    })
  }

  return {
    connected,
    start,
    stop,
  }
}
