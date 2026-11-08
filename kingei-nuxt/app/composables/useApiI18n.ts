import type { ChatType, FriendRequestStatus } from '~/types/api'
import { API_ERROR_CODES, type ApiErrorCode } from '~/types/api'
import { isApiError } from '~/utils/api/error'
import { formatFileSize } from '~/utils/display'

function isApiErrorCodeValue(code: string): code is ApiErrorCode {
  return (API_ERROR_CODES as readonly string[]).includes(code)
}

export function useApiI18n() {
  const { t } = useI18n()
  const { instance } = useInstance()

  function fromError(error: unknown): string {
    if (isApiError(error) && isApiErrorCodeValue(error.code)) {
      if (error.code === 'FileTooLarge') {
        const max = instance.value.max_upload_size
        if (Number.isFinite(max) && max > 0) {
          return t('error.FileTooLargeWithSize', { size: formatFileSize(max) })
        }
      }
      return t(`error.${error.code}`)
    }
    return t('error.InternalServerError')
  }

  function chatType(type: ChatType): string {
    return t(`enum.chatType.${type}`)
  }

  function friendStatus(status: FriendRequestStatus): string {
    return t(`enum.friendStatus.${status}`)
  }

  return {
    fromError,
    chatType,
    friendStatus,
  }
}
