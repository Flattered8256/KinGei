import { ElMessage } from 'element-plus'
import 'element-plus/es/components/message/style/css'

export function useToast() {
  const apiI18n = useApiI18n()

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

  return { success, error }
}
