import type {
  Chat,
  CreateDirectRequest,
  CreateGroupRequest,
  InviteMemberRequest,
  UpdateChatRequest,
} from '~/types/api'
import { apiFetch, apiFetchVoid } from './http'

export function createDirectChat(body: CreateDirectRequest) {
  return apiFetch<Chat>('/chat/direct', {
    method: 'POST',
    body,
  })
}

export function createGroupChat(body: CreateGroupRequest) {
  return apiFetch<Chat>('/chat/group', {
    method: 'POST',
    body,
  })
}

export function listChats() {
  return apiFetch<Chat[]>('/chat')
}

export function getChat(id: number) {
  return apiFetch<Chat>(`/chat/${id}`)
}

export function updateChat(id: number, body: UpdateChatRequest) {
  return apiFetch<Chat>(`/chat/${id}`, {
    method: 'PATCH',
    body,
  })
}

export function inviteChatMember(id: number, body: InviteMemberRequest) {
  return apiFetch<Chat>(`/chat/${id}/member`, {
    method: 'POST',
    body,
  })
}

export function removeChatMember(id: number, userId: string) {
  return apiFetchVoid(`/chat/${id}/member/${userId}`, {
    method: 'DELETE',
  })
}
