import type { Friend, FriendRequest, SendFriendRequest } from '~/types/api'
import { apiFetch, apiFetchVoid } from './http'

export function listFriends() {
  return apiFetch<Friend[]>('/friend')
}

export function deleteFriend(friendId: string) {
  return apiFetchVoid(`/friend/${friendId}`, {
    method: 'DELETE',
  })
}

export function sendFriendRequest(body: SendFriendRequest) {
  return apiFetch<FriendRequest>('/friend/request', {
    method: 'POST',
    body,
  })
}

export function listReceivedFriendRequests() {
  return apiFetch<FriendRequest[]>('/friend/request/received')
}

export function listSentFriendRequests() {
  return apiFetch<FriendRequest[]>('/friend/request/sent')
}

export function acceptFriendRequest(id: number) {
  return apiFetch<FriendRequest>(`/friend/request/${id}/accept`, {
    method: 'PATCH',
  })
}

export function rejectFriendRequest(id: number) {
  return apiFetch<FriendRequest>(`/friend/request/${id}/reject`, {
    method: 'PATCH',
  })
}

export function cancelFriendRequest(id: number) {
  return apiFetch<FriendRequest>(`/friend/request/${id}`, {
    method: 'DELETE',
  })
}
