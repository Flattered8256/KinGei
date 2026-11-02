import type { User } from '~/types/api'
import { refreshSession } from '~/utils/api/http'
import { onSessionChange, restoreSession } from '~/utils/api/session'

const REFRESH_EVERY_MS = 14 * 60 * 1000

export default defineNuxtPlugin(() => {
  const accessToken = useState<string | null>('auth.accessToken', () => null)
  const refreshToken = useState<string | null>('auth.refreshToken', () => null)
  const user = useState<User | null>('auth.user', () => null)

  const apply = restoreSession()
  accessToken.value = apply.accessToken
  refreshToken.value = apply.refreshToken
  user.value = apply.user

  let timer: ReturnType<typeof setInterval> | null = null

  function arm(hasAccess: boolean) {
    if (timer != null) {
      clearInterval(timer)
      timer = null
    }
    if (!hasAccess) {
      return
    }
    timer = setInterval(() => {
      void refreshSession().catch(() => {})
    }, REFRESH_EVERY_MS)
  }

  arm(Boolean(apply.accessToken))

  onSessionChange((session) => {
    accessToken.value = session.accessToken
    refreshToken.value = session.refreshToken
    user.value = session.user
    arm(Boolean(session.accessToken))
  })
})
