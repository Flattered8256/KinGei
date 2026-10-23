import type {
  CreateFileRequest,
  FileResponse,
  PresignDownloadResponse,
  PresignUploadResponse,
} from '~/types/api'
import { ApiError } from './error'
import { apiFetch } from './http'

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

export async function putToPresign(url: string, data: Blob, contentType: string) {
  const response = await fetch(url, {
    method: 'PUT',
    headers: { 'Content-Type': contentType },
    body: data,
  })
  if (!response.ok) {
    throw new ApiError(response.status, 'InternalServerError')
  }
}

const IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif']

export function isImageType(type: string) {
  return IMAGE_TYPES.includes(type)
}

export async function uploadImage(file: File) {
  if (!isImageType(file.type)) {
    throw new ApiError(400, 'InvalidImage')
  }
  const created = await createFile({
    name: file.name,
    content_type: file.type,
    size: file.size,
  })
  await putToPresign(created.upload_url, file, file.type)
  return completeFile(created.id)
}
