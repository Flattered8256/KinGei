import { ElMessage, type MessageHandler } from 'element-plus'
import 'element-plus/es/components/message/style/css'
import KgSpinner from '~/components/KgSpinner.vue'

let sseHandle: MessageHandler | null = null

export function useToast() {
  const apiI18n = useApiI18n()
  const { t } = useI18n()

  function success(message: string) {
    ElMessage({
      type: 'success',
      message,
      grouping: true,
      duration: 2500,
      customClass: 'kg-toast',
    })
  }

  function error(err: unknown) {
    const message = typeof err === 'string' ? err : apiI18n.fromError(err)
    ElMessage({
      type: 'error',
      message,
      grouping: true,
      duration: 3500,
      customClass: 'kg-toast',
    })
  }

  function sse(status: 'idle' | 'connecting' | 'connected' | 'failed') {
    const prev = sseHandle
    sseHandle = null
    prev?.close()
    if (status === 'idle') {
      return
    }

    const handle = ElMessage({
      type: status === 'connected' ? 'success' : status === 'failed' ? 'error' : 'info',
      message: t(`sse.${status}`),
      duration: status === 'connected' ? 1800 : 0,
      grouping: false,
      customClass: 'kg-toast',
      icon: status === 'connecting' ? KgSpinner : undefined,
      onClose: () => {
        if (sseHandle === handle) {
          sseHandle = null
        }
      },
    })
    sseHandle = handle
  }

  return { success, error, sse }
}
