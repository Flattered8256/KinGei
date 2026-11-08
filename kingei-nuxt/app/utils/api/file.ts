import type { FileResponse } from '~/types/api'
import { ApiError } from './error'
import { API_BASE, apiUpload } from './http'

const IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif']
const VIDEO_TYPES = ['video/mp4', 'video/quicktime', 'video/webm']

export type UploadPhase = 'uploading' | 'processing'

export type UploadFileOptions = {
  onProgress?: (percent: number) => void
  onPhase?: (phase: UploadPhase) => void
  signal?: AbortSignal
}

export function downloadFileUrl(id: number) {
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

export function uploadFile(file: File, options: UploadFileOptions = {}) {
  options.onPhase?.('uploading')
  const form = new FormData()
  form.append('name', file.name)
  form.append('file', file, file.name)
  return apiUpload<FileResponse>('/file', form, {
    signal: options.signal,
    onProgress: options.onProgress,
    onUploaded: () => options.onPhase?.('processing'),
  })
}

export async function uploadImage(file: File) {
  if (!isImageType(file.type)) {
    throw new ApiError(400, 'InvalidImage')
  }
  return uploadFile(file)
}
