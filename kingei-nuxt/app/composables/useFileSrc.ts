import type { MaybeRefOrGetter } from 'vue'
import { refreshSession } from '~/utils/api/http'

export function useFileSrc(fileId: MaybeRefOrGetter<number | null | undefined>) {
  const bust = ref(0)
  const config = useRuntimeConfig()
  const apiBase = String(config.public.apiBase)

  watch(
    () => toValue(fileId),
    () => {
      bust.value = 0
    },
  )

  const src = computed(() => {
    const id = toValue(fileId)
    if (id == null) {
      return null
    }
    const path = `${apiBase}/file/${id}/content`
    return bust.value > 0 ? `${path}?r=${bust.value}` : path
  })

  async function reload() {
    try {
      await refreshSession()
    }
    catch {
      // cookie 可能已经有效，仍让图片换 query 再请求
    }
    bust.value += 1
  }

  return { src, reload }
}
