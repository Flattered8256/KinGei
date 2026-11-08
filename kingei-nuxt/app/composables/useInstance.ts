import type { Instance, UpdateInstanceRequest } from '~/types/api'
import { updateInstance } from '~/utils/api/admin'
import { getInstance } from '~/utils/api/instance'

const DEFAULT_INSTANCE: Instance = {
  allow_add_friend: true,
  allow_create_group: true,
  initializing: false,
  invite_only: false,
  limit_message_delete: false,
  // /instance 刷新失败时与后端默认 MAX_UPLOAD_SIZE_MB=100 对齐。
  max_upload_size: 100 * 1024 * 1024,
}

let inflight: Promise<void> | null = null

export function useInstance() {
  const { user } = useAuth()
  const instance = useState<Instance>('instance', () => ({ ...DEFAULT_INSTANCE }))

  const isAdmin = computed(() => user.value?.role === 'Admin')
  const canAddFriend = computed(() => instance.value.allow_add_friend || isAdmin.value)
  const canCreateGroup = computed(() => instance.value.allow_create_group || isAdmin.value)

  function refresh() {
    if (inflight) {
      return inflight
    }
    inflight = getInstance()
      .then((next) => {
        instance.value = next
      })
      .catch(() => {
        // keep last (or default) flags; mutating POSTs still 403
      })
      .finally(() => {
        inflight = null
      })
    return inflight
  }

  async function updateFlags(body: UpdateInstanceRequest) {
    return updateInstance(body)
  }

  return {
    instance,
    isAdmin,
    canAddFriend,
    canCreateGroup,
    refresh,
    updateFlags,
  }
}
