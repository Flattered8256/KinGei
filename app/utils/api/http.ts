import { FetchError } from 'ofetch'
import type { RefreshTokenResponse } from '~/types/api'
import { ApiError } from './error'
import {
  clearSession,
  getAccessToken,
  getRefreshToken,
  setSession,
} from './session'

type ApiFetchOptions = {
  method?: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE'
  body?: object
  query?: Record<string, string | number | undefined>
  headers?: HeadersInit
}

const REFRESH_PATH = '/user/refresh'
const SKIP_REFRESH_PATHS = ['/user/login', '/user/register', REFRESH_PATH]

let apiBase = '/api'
let rawFetch: ReturnType<typeof $fetch.create> | null = null
let refreshPromise: Promise<void> | null = null

function request<T>(path: string, options: ApiFetchOptions) {
  return getRawFetch()<T>(path, options as never)
}

function getRawFetch() {
  if (!rawFetch) {
    const config = useRuntimeConfig()
    apiBase = String(config.public.apiBase)
    rawFetch = $fetch.create({
      baseURL: apiBase,
      onRequest({ options }) {
        const token = getAccessToken()
        if (!token) {
          return
        }
        const headers = new Headers(options.headers as HeadersInit | undefined)
        headers.set('Authorization', `Bearer ${token}`)
        options.headers = headers
      },
    })
  }
  return rawFetch
}

export async function apiFetch<T>(
  path: string,
  options: ApiFetchOptions = {},
): Promise<T> {
  try {
    return await request<T>(path, options)
  }
  catch (error) {
    if (shouldRefresh(error, path)) {
      await refreshSession()
      try {
        return await request<T>(path, options)
      }
      catch (retryError) {
        throw toApiError(retryError)
      }
    }
    throw toApiError(error)
  }
}

export async function apiFetchVoid(
  path: string,
  options: ApiFetchOptions = {},
): Promise<void> {
  await apiFetch<unknown>(path, options)
}

function shouldRefresh(error: unknown, path: string): boolean {
  if (SKIP_REFRESH_PATHS.some(skip => path === skip || path.startsWith(`${skip}?`))) {
    return false
  }
  if (!(error instanceof FetchError) || error.status !== 401) {
    return false
  }
  if (!getRefreshToken()) {
    return false
  }
  const code = readErrorCode(error)
  return code === 'InvalidToken' || code === 'InvalidCredentials'
}

export async function refreshSession(): Promise<void> {
  if (!refreshPromise) {
    refreshPromise = doRefresh().finally(() => {
      refreshPromise = null
    })
  }
  await refreshPromise
}

async function doRefresh(): Promise<void> {
  const refreshToken = getRefreshToken()
  if (!refreshToken) {
    clearSession()
    throw new ApiError(401, 'InvalidToken')
  }

  try {
    const result = await $fetch<RefreshTokenResponse>(REFRESH_PATH, {
      baseURL: apiBase,
      method: 'POST',
      body: { refresh_token: refreshToken },
    })
    setSession({
      accessToken: result.access_token,
      refreshToken: result.refresh_token,
    })
  }
  catch (error) {
    clearSession()
    throw toApiError(error)
  }
}

function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) {
    return error
  }
  if (error instanceof FetchError) {
    return new ApiError(error.status ?? 500, readErrorCode(error))
  }
  return new ApiError(500, 'InternalServerError')
}

function readErrorCode(error: FetchError): string {
  const data = error.data as { code?: unknown } | undefined
  return typeof data?.code === 'string' ? data.code : 'InternalServerError'
}
