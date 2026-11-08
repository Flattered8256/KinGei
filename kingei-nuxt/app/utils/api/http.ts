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

export const API_BASE = '/api'
const REFRESH_PATH = '/user/refresh'
const SKIP_REFRESH_PATHS = ['/user/login', '/user/register', '/user/logout', REFRESH_PATH]

export type UploadOptions = {
  onProgress?: (percent: number) => void
  onUploaded?: () => void
  signal?: AbortSignal
}

let rawFetch: ReturnType<typeof $fetch.create> | null = null
let refreshPromise: Promise<void> | null = null

function request<T>(path: string, options: ApiFetchOptions) {
  return getRawFetch()<T>(path, options as never)
}

function getRawFetch() {
  if (!rawFetch) {
    rawFetch = $fetch.create({
      baseURL: API_BASE,
      credentials: 'include',
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

// 上传需要进度回调，ofetch 拿不到；这里用 XHR 但复用同一套 token 与刷新逻辑。
export async function apiUpload<T>(
  path: string,
  form: FormData,
  options: UploadOptions = {},
): Promise<T> {
  try {
    return await sendUpload<T>(path, form, options)
  }
  catch (error) {
    if (shouldRefreshUpload(error)) {
      await refreshSession()
      return await sendUpload<T>(path, form, options)
    }
    throw error
  }
}

function shouldRefreshUpload(error: unknown): boolean {
  if (!(error instanceof ApiError) || error.status !== 401 || !getRefreshToken()) {
    return false
  }
  return error.code === 'InvalidToken' || error.code === 'InvalidCredentials'
}

function sendUpload<T>(
  path: string,
  form: FormData,
  options: UploadOptions,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    if (options.signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }

    const xhr = new XMLHttpRequest()
    xhr.open('POST', `${API_BASE}${path}`)
    xhr.withCredentials = true

    const token = getAccessToken()
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    }

    xhr.upload.onprogress = (event) => {
      if (!event.lengthComputable || !options.onProgress) {
        return
      }
      options.onProgress(Math.round((event.loaded / event.total) * 100))
    }

    // 请求体发完只代表服务端开始探测/出图，前端据此切换阶段。
    xhr.upload.onload = () => options.onUploaded?.()

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const body = JSON.parse(xhr.responseText) as T
          options.onProgress?.(100)
          resolve(body)
        }
        catch {
          reject(new ApiError(500, 'InternalServerError'))
        }
        return
      }
      reject(new ApiError(xhr.status, readUploadCode(xhr.responseText)))
    }

    xhr.onerror = () => {
      reject(new ApiError(500, 'InternalServerError'))
    }

    xhr.onabort = () => {
      reject(new DOMException('Aborted', 'AbortError'))
    }

    const onAbort = () => xhr.abort()
    options.signal?.addEventListener('abort', onAbort, { once: true })
    xhr.addEventListener('loadend', () => {
      options.signal?.removeEventListener('abort', onAbort)
    })

    xhr.send(form)
  })
}

function readUploadCode(text: string): string {
  try {
    const data = JSON.parse(text) as { code?: unknown }
    return typeof data.code === 'string' ? data.code : 'InternalServerError'
  }
  catch {
    return 'InternalServerError'
  }
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
      baseURL: API_BASE,
      method: 'POST',
      credentials: 'include',
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
