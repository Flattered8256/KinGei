import type {
  ChangePasswordRequest,
  CreateUserRequest,
  LoginRequest,
  UpdateUserRequest,
  User,
} from '~/types/api'
import { clearSession, getSession, setSession, setUser } from '~/utils/api/session'
import {
  changePassword as changePasswordRequest,
  loginUser,
  registerUser,
  updateUser as updateUserRequest,
} from '~/utils/api/user'

export function useAuth() {
  const accessToken = useState<string | null>('auth.accessToken', () => getSession().accessToken)
  const refreshToken = useState<string | null>('auth.refreshToken', () => getSession().refreshToken)
  const user = useState<User | null>('auth.user', () => getSession().user)
  const isAuthenticated = computed(() => Boolean(accessToken.value))

  async function register(body: CreateUserRequest) {
    return registerUser(body)
  }

  async function login(body: LoginRequest) {
    const result = await loginUser(body)
    setSession({
      accessToken: result.access_token,
      refreshToken: result.refresh_token,
      user: result.user,
    })
    return result.user
  }

  function logout() {
    clearSession()
  }

  async function updateProfile(body: UpdateUserRequest) {
    const next = await updateUserRequest(body)
    setUser(next)
    return next
  }

  async function updatePassword(body: ChangePasswordRequest) {
    const next = await changePasswordRequest(body)
    setUser(next)
    return next
  }

  return {
    accessToken,
    refreshToken,
    user,
    isAuthenticated,
    register,
    login,
    logout,
    updateProfile,
    updatePassword,
  }
}
