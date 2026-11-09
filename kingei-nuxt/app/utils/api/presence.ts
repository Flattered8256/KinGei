import type { PresenceChangedEvent } from '~/types/api'
import { apiFetch } from './http'

export function getPresence() {
  return apiFetch<PresenceChangedEvent[]>('/presence')
}
