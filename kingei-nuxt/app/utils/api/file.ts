import type { FileResponse } from '~/types/api'
import { ApiError } from './error'
import { refreshSession } from './http'
import { getAccessToken, getRefreshToken } from './session'

const API_BASE = '/api'
const IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif']
const VIDEO_TYPES = ['video/mp4', 'video/quicktime', 'video/webm']

export type UploadPhase = 'uploading' | 'processing'

export type UploadFileOptions = {
  onProgress?: (percent: number) => void
  onPhase?: (phase: UploadPhase) => void
  signal?: AbortSignal
}

export function fileDownloadUrl(id: number) {
  return `${API_BASE}/file/${id}/download`
}

function mimeOf(type: string) {
  return type.trim().toLowerCase()
}

export function isImageType(type: string) {
  return IMAGE_TYPES.includes(mimeOf(type))
}

export function isVideoType(type: string) {
  return VIDEO_TYPES.includes(mimeOf(type))
}

export function isAbortError(error: unknown) {
  return error instanceof DOMException && error.name === 'AbortError'
}

function parseUpload(xhr: XMLHttpRequest): FileResponse | null {
  try {
    const data = JSON.parse(xhr.responseText) as Partial<FileResponse>
    return typeof data.id === 'number' ? (data as FileResponse) : null
  }
  catch {
    return null
  }
}

function uploadError(xhr: XMLHttpRequest): ApiError {
  const status = xhr.status >= 400 ? xhr.status : 500
  if (status === 413) {
    return new ApiError(status, 'FileTooLarge')
  }
  try {
    const data = JSON.parse(xhr.responseText) as { code?: unknown }
    if (typeof data.code === 'string') {
      return new ApiError(status, data.code)
    }
  }
  catch {
    // 代理或网关可能返回非 JSON 错误页
  }
  return new ApiError(status, 'InternalServerError')
}

function uploadOnce(file: File, options: UploadFileOptions) {
  return new Promise<FileResponse>((resolve, reject) => {
    if (options.signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }

    const xhr = new XMLHttpRequest()
    xhr.open('POST', `${API_BASE}/file`)
    xhr.withCredentials = true

    let done = false
    const token = getAccessToken()
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`)
    }

    xhr.upload.onprogress = (event) => {
      if (done || !event.lengthComputable || !options.onProgress) {
        return
      }
      options.onProgress(Math.min(99, Math.round((event.loaded / event.total) * 100)))
    }

    xhr.upload.onload = () => {
      if (done) {
        return
      }
      options.onProgress?.(100)
      options.onPhase?.('processing')
    }

    xhr.onload = () => {
      done = true
      if (xhr.status >= 200 && xhr.status < 300) {
        const uploaded = parseUpload(xhr)
        if (uploaded) {
          resolve(uploaded)
          return
        }
        reject(new ApiError(500, 'InternalServerError'))
        return
      }
      reject(uploadError(xhr))
    }

    xhr.onerror = () => {
      done = true
      reject(new ApiError(500, 'InternalServerError'))
    }

    xhr.onabort = () => {
      done = true
      reject(new DOMException('Aborted', 'AbortError'))
    }

    const onAbort = () => xhr.abort()
    options.signal?.addEventListener('abort', onAbort, { once: true })
    xhr.addEventListener('loadend', () => {
      options.signal?.removeEventListener('abort', onAbort)
    })

    const form = new FormData()
    form.append('file', file, file.name || 'file')
    xhr.send(form)
  })
}

function shouldRefresh(error: unknown, signal?: AbortSignal) {
  return error instanceof ApiError
    && error.status === 401
    && Boolean(getRefreshToken())
    && !signal?.aborted
}

export async function uploadFile(
  file: File,
  options: UploadFileOptions = {},
): Promise<FileResponse> {
  options.onPhase?.('uploading')
  try {
    return await uploadOnce(file, options)
  }
  catch (error) {
    if (!shouldRefresh(error, options.signal)) {
      throw error
    }
    try {
      await refreshSession()
    }
    catch {
      throw error
    }
    if (options.signal?.aborted) {
      throw new DOMException('Aborted', 'AbortError')
    }
    options.onProgress?.(0)
    options.onPhase?.('uploading')
    return uploadOnce(file, options)
  }
}

export async function uploadImage(file: File) {
  if (!isImageType(file.type)) {
    throw new ApiError(400, 'InvalidImage')
  }
  return uploadFile(file)
}
