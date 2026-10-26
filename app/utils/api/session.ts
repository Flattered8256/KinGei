import type { User } from '~/types/api'

const ACCESS_TOKEN_KEY = 'kingei.access_token'
const REFRESH_TOKEN_KEY = 'kingei.refresh_token'
const USER_KEY = 'kingei.user'
const ACCESS_COOKIE = 'kingei.access_token'
const ACCESS_COOKIE_MAX_AGE = 900

export interface AuthSession {
  accessToken: string | null
  refreshToken: string | null
  user: User | null
}

const session: AuthSession = {
  accessToken: null,
  refreshToken: null,
  user: null,
}

type SessionListener = (snapshot: AuthSession) => void
const listeners = new Set<SessionListener>()

export function getSession(): AuthSession {
  return session
}

export function getAccessToken(): string | null {
  return session.accessToken
}

export function getRefreshToken(): string | null {
  return session.refreshToken
}

export function onSessionChange(listener: SessionListener): () => void {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

export function setSession(next: {
  accessToken: string
  refreshToken: string
  user?: User | null
}): void {
  session.accessToken = next.accessToken
  session.refreshToken = next.refreshToken
  if (next.user !== undefined) {
    session.user = next.user
  }
  persist()
  notify()
}

export function setUser(user: User | null): void {
  session.user = user
  persist()
  notify()
}

export function clearSession(): void {
  session.accessToken = null
  session.refreshToken = null
  session.user = null
  persist()
  notify()
}

export function restoreSession(): AuthSession {
  if (!import.meta.client) {
    return session
  }

  session.accessToken = localStorage.getItem(ACCESS_TOKEN_KEY)
  session.refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY)
  session.user = readStoredUser()
  writeAccessCookie(session.accessToken)
  notify()
  return session
}

function notify(): void {
  const snapshot: AuthSession = {
    accessToken: session.accessToken,
    refreshToken: session.refreshToken,
    user: session.user,
  }
  for (const listener of listeners) {
    listener(snapshot)
  }
}

function persist(): void {
  if (!import.meta.client) {
    return
  }

  writeKey(ACCESS_TOKEN_KEY, session.accessToken)
  writeKey(REFRESH_TOKEN_KEY, session.refreshToken)
  writeAccessCookie(session.accessToken)
  if (session.user) {
    localStorage.setItem(USER_KEY, JSON.stringify(session.user))
  }
  else {
    localStorage.removeItem(USER_KEY)
  }
}

function writeAccessCookie(token: string | null) {
  if (!import.meta.client) {
    return
  }
  if (token) {
    const secure = location.protocol === 'https:' ? '; Secure' : ''
    document.cookie = `${ACCESS_COOKIE}=${token}; Path=/; SameSite=Lax; Max-Age=${ACCESS_COOKIE_MAX_AGE}${secure}`
  }
  else {
    document.cookie = `${ACCESS_COOKIE}=; Path=/; SameSite=Lax; Max-Age=0`
  }
}

function writeKey(key: string, value: string | null): void {
  if (value) {
    localStorage.setItem(key, value)
  }
  else {
    localStorage.removeItem(key)
  }
}

function readStoredUser(): User | null {
  const raw = localStorage.getItem(USER_KEY)
  if (!raw) {
    return null
  }

  try {
    return JSON.parse(raw) as User
  }
  catch {
    localStorage.removeItem(USER_KEY)
    return null
  }
}
