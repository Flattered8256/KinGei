import type { ChatType, FriendRequestStatus } from '~/types/api'
import { API_ERROR_CODES, type ApiErrorCode } from '~/types/api'
import { isApiError } from '~/utils/api/error'

function isApiErrorCodeValue(code: string): code is ApiErrorCode {
  return (API_ERROR_CODES as readonly string[]).includes(code)
}

export function useApiI18n() {
  const { t } = useI18n()

  function fromError(error: unknown): string {
    if (isApiError(error) && isApiErrorCodeValue(error.code)) {
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
