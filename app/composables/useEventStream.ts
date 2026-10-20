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
    connection = connectEventStream({
      onEvent: options.onEvent,
      onReconnect: options.onReconnect,
    })
    connected.value = true
  }

  function stop() {
    connection?.stop()
    connection = null
    connected.value = false
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

    onBeforeUnmount(() => {
      stop()
    })
  }

  return {
    connected,
    start,
    stop,
  }
}
