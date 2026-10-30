import type { Instance, Invite, UpdateInstanceRequest, User } from '~/types/api'
import { apiFetch, apiFetchVoid } from './http'

export function listAdminUsers() {
  return apiFetch<User[]>('/admin/user')
}

export function deleteAdminUser(id: string) {
  return apiFetchVoid(`/admin/user/${id}`, {
    method: 'DELETE',
  })
}

export function updateInstance(body: UpdateInstanceRequest) {
  return apiFetch<Instance>('/admin/instance', {
    method: 'PATCH',
    body,
  })
}

export function getInvite() {
  return apiFetch<Invite>('/admin/invite')
}

export function regenerateInvite() {
  return apiFetch<Invite>('/admin/invite', {
    method: 'POST',
  })
}
