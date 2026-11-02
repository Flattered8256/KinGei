import type { ClientEvent, ClientEventName } from '~/types/api'
import { CLIENT_EVENT_NAMES } from '~/types/api'
import { ApiError } from './error'
import { refreshSession } from './http'
import { getAccessToken } from './session'

const KNOWN_EVENTS = new Set<string>(CLIENT_EVENT_NAMES)
const MAX_BACKOFF_MS = 15_000
const IDLE_MS = 45_000

export interface EventStreamHandlers {
  onEvent: (event: ClientEvent) => void
  onReconnect?: () => void
  onConnecting?: () => void
  onOpen?: () => void
  onDisconnect?: () => void
  onClose?: () => void
}

export interface EventStreamConnection {
  stop: () => void
  wake: () => void
}

export function connectEventStream(handlers: EventStreamHandlers): EventStreamConnection {
  const apiBase = String(useRuntimeConfig().public.apiBase)
  let stopped = false
  let skipDelay = false
  let attemptAbort: AbortController | null = null
  let delayAbort: AbortController | null = null

  function interrupt() {
    attemptAbort?.abort()
    delayAbort?.abort()
  }

  function wake() {
    if (stopped) {
      return
    }
    skipDelay = true
    interrupt()
  }

  function stop() {
    if (stopped) {
      return
    }
    stopped = true
    interrupt()
  }

  const loop = async () => {
    let attempt = 0
    let openedOnce = false

    try {
      while (!stopped) {
        const ac = new AbortController()
        attemptAbort = ac
        await Promise.resolve()
        if (stopped) {
          break
        }
        handlers.onConnecting?.()

        try {
          await consumeEventStream(apiBase, handlers.onEvent, ac.signal, () => {
            handlers.onOpen?.()
            if (openedOnce) {
              handlers.onReconnect?.()
            }
            openedOnce = true
            attempt = 0
          })
          attempt = 0
          if (!stopped) {
            handlers.onDisconnect?.()
          }
        }
        catch (error) {
          if (stopped) {
            break
          }
          if (isAbortError(error)) {
            skipDelay = true
            attempt = 0
          }
          else if (error instanceof ApiError && error.status === 401) {
            try {
              await refreshSession()
              if (stopped) {
                break
              }
              skipDelay = true
              attempt = 0
            }
            catch {
              break
            }
          }
          else {
            handlers.onDisconnect?.()
          }
        }

        if (stopped) {
          break
        }

        if (skipDelay) {
          skipDelay = false
          continue
        }

        const delay = Math.min(1000 * 2 ** attempt, MAX_BACKOFF_MS)
        attempt += 1
        delayAbort = new AbortController()
        await sleep(delay, delayAbort.signal)
        delayAbort = null
        if (skipDelay) {
          skipDelay = false
        }
      }
    }
    finally {
      handlers.onClose?.()
    }
  }

  void loop()

  return { stop, wake }
}

async function consumeEventStream(
  apiBase: string,
  onEvent: (event: ClientEvent) => void,
  signal: AbortSignal,
  onOpen: () => void,
): Promise<void> {
  const token = getAccessToken()
  if (!token) {
    throw new ApiError(401, 'InvalidCredentials')
  }

  const local = new AbortController()
  const onParentAbort = () => local.abort()
  if (signal.aborted) {
    local.abort()
  }
  else {
    signal.addEventListener('abort', onParentAbort, { once: true })
  }

  let idleTimer: ReturnType<typeof setTimeout> | undefined
  const clearIdle = () => {
    if (idleTimer !== undefined) {
      clearTimeout(idleTimer)
      idleTimer = undefined
    }
  }
  const armIdle = () => {
    clearIdle()
    idleTimer = setTimeout(() => local.abort(), IDLE_MS)
  }

  try {
    const response = await fetch(`${apiBase}/event`, {
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: 'text/event-stream',
        'Cache-Control': 'no-store',
      },
      credentials: 'include',
      cache: 'no-store',
      signal: local.signal,
    })

    if (!response.ok) {
      throw await errorFromResponse(response)
    }
    if (!response.body) {
      throw new ApiError(500, 'InternalServerError')
    }

    onOpen()
    armIdle()

    const reader = response.body.getReader()
    const decoder = new TextDecoder()
    let buffer = ''

    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) {
          if (local.signal.aborted) {
            throw new DOMException('The operation was aborted.', 'AbortError')
          }
          break
        }
        armIdle()
        buffer += decoder.decode(value, { stream: true })
        const parsed = takeSseBlocks(buffer)
        buffer = parsed.rest
        for (const event of parsed.events) {
          onEvent(event)
        }
      }
    }
    finally {
      try {
        reader.releaseLock()
      }
      catch {
        // abort() already closed the reader
      }
    }
  }
  finally {
    clearIdle()
    signal.removeEventListener('abort', onParentAbort)
  }
}

function takeSseBlocks(buffer: string): { events: ClientEvent[]; rest: string } {
  const events: ClientEvent[] = []
  const parts = buffer.split(/\r?\n\r?\n/)
  const rest = parts.pop() ?? ''

  for (const block of parts) {
    const event = parseSseBlock(block)
    if (event) {
      events.push(event)
    }
  }

  return { events, rest }
}

function parseSseBlock(block: string): ClientEvent | null {
  let eventName = 'message'
  const dataLines: string[] = []

  for (const rawLine of block.split(/\r?\n/)) {
    if (!rawLine || rawLine.startsWith(':')) {
      continue
    }
    const separator = rawLine.indexOf(':')
    const field = separator === -1 ? rawLine : rawLine.slice(0, separator)
    let value = separator === -1 ? '' : rawLine.slice(separator + 1)
    if (value.startsWith(' ')) {
      value = value.slice(1)
    }
    if (field === 'event') {
      eventName = value
    }
    else if (field === 'data') {
      dataLines.push(value)
    }
  }

  if (dataLines.length === 0 || !KNOWN_EVENTS.has(eventName)) {
    return null
  }

  try {
    return {
      event: eventName as ClientEventName,
      data: JSON.parse(dataLines.join('\n')),
    } as ClientEvent
  }
  catch {
    return null
  }
}

async function errorFromResponse(response: Response): Promise<ApiError> {
  try {
    const data = (await response.json()) as { code?: string }
    return new ApiError(response.status, data.code ?? 'InternalServerError')
  }
  catch {
    return new ApiError(response.status, 'InternalServerError')
  }
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException
    ? error.name === 'AbortError'
    : error instanceof Error && error.name === 'AbortError'
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  if (signal.aborted) {
    return Promise.resolve()
  }

  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      signal.removeEventListener('abort', onAbort)
      resolve()
    }, ms)
    const onAbort = () => {
      clearTimeout(timer)
      resolve()
    }
    signal.addEventListener('abort', onAbort, { once: true })
  })
}
