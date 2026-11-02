import type { Instance } from '~/types/api'
import { apiFetch } from './http'

export function getInstance() {
  return apiFetch<Instance>('/instance')
}
