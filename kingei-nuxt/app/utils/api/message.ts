import type {
  ListMessageQuery,
  Message,
  ReadCursor,
  SendMessageRequest,
  UpdateReadCursorRequest,
} from '~/types/api'
import { apiFetch } from './http'

export function listMessages(chatId: number, query: ListMessageQuery = {}) {
  return apiFetch<Message[]>(`/chat/${chatId}/message`, {
    query: {
      before: query.before,
      after: query.after,
      limit: query.limit,
    },
  })
}

export function sendMessage(chatId: number, body: SendMessageRequest) {
  return apiFetch<Message>(`/chat/${chatId}/message`, {
    method: 'POST',
    body,
  })
}

export function updateReadCursor(chatId: number, body: UpdateReadCursorRequest) {
  return apiFetch<ReadCursor>(`/chat/${chatId}/read`, {
    method: 'PATCH',
    body,
  })
}
