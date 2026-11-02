import type {
  CreateFileRequest,
  FileResponse,
  PresignDownloadResponse,
  PresignUploadResponse,
} from '~/types/api'
import { ApiError } from './error'
import { apiFetch } from './http'

const IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif']
const VIDEO_TYPES = ['video/mp4', 'video/quicktime', 'video/webm']

export type UploadPhase = 'uploading' | 'processing'

export type UploadFileOptions = {
  onProgress?: (percent: number) => void
  onPhase?: (phase: UploadPhase) => void
  signal?: AbortSignal
}

export function createFile(body: CreateFileRequest) {
  return apiFetch<PresignUploadResponse>('/file', {
    method: 'POST',
    body,
  })
}

export function completeFile(id: number) {
  return apiFetch<FileResponse>(`/file/${id}/complete`, {
    method: 'POST',
  })
}

export function downloadFile(id: number) {
  return apiFetch<PresignDownloadResponse>(`/file/${id}/download`)
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

export function putToPresign(
  url: string,
  data: Blob,
  contentType: string,
  onProgress?: (percent: number) => void,
  signal?: AbortSignal,
) {
  return new Promise<void>((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }

    const xhr = new XMLHttpRequest()
    xhr.open('PUT', url)
    xhr.setRequestHeader('Content-Type', contentType)

    xhr.upload.onprogress = (event) => {
      if (!event.lengthComputable || !onProgress) {
        return
      }
      onProgress(Math.round((event.loaded / event.total) * 100))
    }

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        onProgress?.(100)
        resolve()
        return
      }
      reject(new ApiError(xhr.status, 'InternalServerError'))
    }

    xhr.onerror = () => {
      reject(new ApiError(500, 'InternalServerError'))
    }

    xhr.onabort = () => {
      reject(new DOMException('Aborted', 'AbortError'))
    }

    const onAbort = () => xhr.abort()
    signal?.addEventListener('abort', onAbort, { once: true })
    xhr.addEventListener('loadend', () => {
      signal?.removeEventListener('abort', onAbort)
    })

    xhr.send(data)
  })
}

export async function uploadFile(file: File, options: UploadFileOptions = {}) {
  const contentType = file.type.trim() || 'application/octet-stream'
  options.onPhase?.('uploading')
  const created = await createFile({
    name: file.name,
    content_type: contentType,
    size: file.size,
  })
  if (options.signal?.aborted) {
    throw new DOMException('Aborted', 'AbortError')
  }
  await putToPresign(
    created.upload_url,
    file,
    contentType,
    options.onProgress,
    options.signal,
  )
  options.onPhase?.('processing')
  return completeFile(created.id)
}

export async function uploadImage(file: File) {
  if (!isImageType(file.type)) {
    throw new ApiError(400, 'InvalidImage')
  }
  return uploadFile(file)
}
