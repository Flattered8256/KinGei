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

export function sendMessage(
  chatId: number,
  body: SendMessageRequest,
  replyToId?: number,
) {
  const path = replyToId != null
    ? `/chat/${chatId}/message/${replyToId}`
    : `/chat/${chatId}/message`
  return apiFetch<Message>(path, {
    method: 'POST',
    body,
  })
}

export function deleteMessage(chatId: number, messageId: number) {
  return apiFetch<Message>(`/chat/${chatId}/message/${messageId}`, {
    method: 'DELETE',
  })
}

export function updateReadCursor(chatId: number, body: UpdateReadCursorRequest) {
  return apiFetch<ReadCursor>(`/chat/${chatId}/read`, {
    method: 'PATCH',
    body,
  })
}
