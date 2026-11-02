import type { ClientEvent } from '~/types/api'
import {
  connectEventStream,
  type EventStreamConnection,
} from '~/utils/api/event'

export type SseStatus = 'idle' | 'connecting' | 'connected' | 'failed'

export interface UseEventStreamOptions {
  onEvent: (event: ClientEvent) => void
  onReconnect?: () => void
  immediate?: boolean
}

export function useSseStatus() {
  return useState<SseStatus>('sse.status', () => 'idle')
}

export function useEventStream(options: UseEventStreamOptions) {
  const { isAuthenticated } = useAuth()
  const toast = useToast()
  const status = useSseStatus()
  const connected = computed(() => status.value === 'connected')
  let connection: EventStreamConnection | null = null

  function setStatus(next: SseStatus) {
    status.value = next
  }

  function start() {
    if (!import.meta.client || connection || !isAuthenticated.value) {
      return
    }
    let conn: EventStreamConnection | null = null
    conn = connectEventStream({
      onEvent: options.onEvent,
      onReconnect: options.onReconnect,
      onConnecting: () => {
        if (connection === conn) {
          setStatus('connecting')
        }
      },
      onOpen: () => {
        if (connection === conn) {
          setStatus('connected')
        }
      },
      onDisconnect: () => {
        if (connection === conn) {
          setStatus('failed')
        }
      },
      onClose: () => {
        if (connection === conn) {
          connection = null
          setStatus('idle')
        }
      },
    })
    connection = conn
  }

  function stop() {
    const conn = connection
    connection = null
    setStatus('idle')
    conn?.stop()
  }

  function wake() {
    connection?.wake()
  }

  if (import.meta.client) {
    watch(status, (next) => {
      toast.sse(next)
    }, { immediate: true })

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
    status,
    start,
    stop,
  }
}
