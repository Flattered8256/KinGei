import type { MaybeRefOrGetter } from 'vue'
import { downloadFile } from '~/utils/api/file'

type CacheEntry = {
  url: string
  expiresAt: number
}

const inflight = new Map<number, Promise<string | null>>()
const SKEW_MS = 60_000

export function useAvatarSrc(fileId: MaybeRefOrGetter<number | null | undefined>) {
  const cache = useState<Record<number, CacheEntry>>('avatars.byFileId', () => ({}))

  const src = computed(() => {
    const id = toValue(fileId)
    if (id == null) {
      return null
    }
    const hit = cache.value[id]
    if (!hit || hit.expiresAt <= Date.now()) {
      return null
    }
    return hit.url
  })

  async function load(id: number) {
    const hit = cache.value[id]
    if (hit && hit.expiresAt > Date.now() + SKEW_MS) {
      return hit.url
    }
    const pending = inflight.get(id)
    if (pending) {
      return pending
    }
    const task = downloadFile(id)
      .then((result) => {
        const expiresAt = Date.parse(result.expires_at)
        cache.value = {
          ...cache.value,
          [id]: {
            url: result.download_url,
            expiresAt: Number.isFinite(expiresAt) ? expiresAt : Date.now() + 14 * 60_000,
          },
        }
        return result.download_url
      })
      .catch(() => null)
      .finally(() => {
        inflight.delete(id)
      })
    inflight.set(id, task)
    return task
  }

  watch(
    () => toValue(fileId),
    (id) => {
      if (id != null) {
        void load(id)
      }
    },
    { immediate: true },
  )

  function reload() {
    const id = toValue(fileId)
    if (id == null) {
      return
    }
    const next = { ...cache.value }
    delete next[id]
    cache.value = next
    void load(id)
  }

  return { src, reload }
}
