import type { User } from '~/types/api'
import { onSessionChange, restoreSession } from '~/utils/api/session'

export default defineNuxtPlugin(() => {
  const accessToken = useState<string | null>('auth.accessToken', () => null)
  const refreshToken = useState<string | null>('auth.refreshToken', () => null)
  const user = useState<User | null>('auth.user', () => null)

  const apply = restoreSession()
  accessToken.value = apply.accessToken
  refreshToken.value = apply.refreshToken
  user.value = apply.user

  onSessionChange((session) => {
    accessToken.value = session.accessToken
    refreshToken.value = session.refreshToken
    user.value = session.user
  })
})
