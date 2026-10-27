import type { ClientEvent, ClientEventName, RefreshTokenResponse } from '~/types/api'
import { CLIENT_EVENT_NAMES } from '~/types/api'
import { ApiError } from './error'
import {
  clearSession,
  getAccessToken,
  getRefreshToken,
  setSession,
} from './session'

const KNOWN_EVENTS = new Set<string>(CLIENT_EVENT_NAMES)
const MAX_BACKOFF_MS = 15_000

export interface EventStreamHandlers {
  onEvent: (event: ClientEvent) => void
  onReconnect?: () => void
}

export interface EventStreamConnection {
  stop: () => void
}

export function connectEventStream(handlers: EventStreamHandlers): EventStreamConnection {
  const apiBase = String(useRuntimeConfig().public.apiBase)
  const abort = new AbortController()
  let stopped = false

  const loop = async () => {
    let attempt = 0
    let hasConnected = false

    while (!stopped) {
      try {
        await consumeEventStream(apiBase, handlers.onEvent, abort.signal)
        attempt = 0
        hasConnected = true
      }
      catch (error) {
        if (stopped || abort.signal.aborted) {
          break
        }
        if (error instanceof ApiError && error.status === 401) {
          const refreshed = await tryRefresh(apiBase)
          if (!refreshed) {
            break
          }
          continue
        }
      }

      if (stopped || abort.signal.aborted) {
        break
      }

      if (hasConnected) {
        handlers.onReconnect?.()
        hasConnected = false
      }

      const delay = Math.min(1000 * 2 ** attempt, MAX_BACKOFF_MS)
      attempt += 1
      await sleep(delay, abort.signal)
    }
  }

  void loop()

  return {
    stop() {
      stopped = true
      abort.abort()
    },
  }
}

async function consumeEventStream(
  apiBase: string,
  onEvent: (event: ClientEvent) => void,
  signal: AbortSignal,
): Promise<void> {
  const token = getAccessToken()
  if (!token) {
    throw new ApiError(401, 'InvalidCredentials')
  }

  const response = await fetch(`${apiBase}/event`, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: 'text/event-stream',
    },
    credentials: 'include',
    signal,
  })

  if (!response.ok) {
    throw await errorFromResponse(response)
  }
  if (!response.body) {
    throw new ApiError(500, 'InternalServerError')
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''

  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) {
        break
      }
      buffer += decoder.decode(value, { stream: true })
      const parsed = takeSseBlocks(buffer)
      buffer = parsed.rest
      for (const event of parsed.events) {
        onEvent(event)
      }
    }
  }
  finally {
    reader.releaseLock()
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

async function tryRefresh(apiBase: string): Promise<boolean> {
  const refreshToken = getRefreshToken()
  if (!refreshToken) {
    clearSession()
    return false
  }

  try {
    const result = await $fetch<RefreshTokenResponse>('/user/refresh', {
      baseURL: apiBase,
      method: 'POST',
      credentials: 'include',
      body: { refresh_token: refreshToken },
    })
    setSession({
      accessToken: result.access_token,
      refreshToken: result.refresh_token,
    })
    return true
  }
  catch {
    clearSession()
    return false
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
