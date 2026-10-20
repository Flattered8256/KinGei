import type {
  ChangePasswordRequest,
  CreateUserRequest,
  LoginRequest,
  LoginResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
  UpdateUserRequest,
  User,
} from '~/types/api'
import { apiFetch } from './http'

export function registerUser(body: CreateUserRequest) {
  return apiFetch<User>('/user/register', {
    method: 'POST',
    body,
  })
}

export function loginUser(body: LoginRequest) {
  return apiFetch<LoginResponse>('/user/login', {
    method: 'POST',
    body,
  })
}

export function refreshTokens(body: RefreshTokenRequest) {
  return apiFetch<RefreshTokenResponse>('/user/refresh', {
    method: 'POST',
    body,
  })
}

export function changePassword(body: ChangePasswordRequest) {
  return apiFetch<User>('/user/change_password', {
    method: 'PATCH',
    body,
  })
}

export function updateUser(body: UpdateUserRequest) {
  return apiFetch<User>('/user/update', {
    method: 'PATCH',
    body,
  })
}

export function findUser(email: string) {
  return apiFetch<User>(`/user/${encodeURIComponent(email)}`)
}
