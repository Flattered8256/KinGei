import { ApiError } from '~/utils/api/error'
import {
  uploadFile,
  uploadImage as uploadImageRequest,
  type UploadFileOptions,
} from '~/utils/api/file'

export function useFileUpload() {
  const { instance } = useInstance()

  function ensureWithinLimit(file: File) {
    const max = instance.value.max_upload_size
    if (Number.isFinite(max) && max > 0 && file.size > max) {
      throw new ApiError(413, 'FileTooLarge')
    }
  }

  function upload(file: File, options: UploadFileOptions = {}) {
    ensureWithinLimit(file)
    return uploadFile(file, options)
  }

  function uploadImage(file: File) {
    ensureWithinLimit(file)
    return uploadImageRequest(file)
  }

  return { upload, uploadImage }
}
