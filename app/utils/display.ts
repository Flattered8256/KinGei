import type { Chat, User } from '~/types/api'

const AVATAR_COLORS = [
  '#0e7090',
  '#d97706',
  '#7c3aed',
  '#dc2626',
  '#059669',
  '#2563eb',
  '#db2777',
  '#0d9488',
]

export function initials(name: string | null | undefined): string {
  const value = (name ?? '').trim()
  if (!value) {
    return '?'
  }
  const chars = Array.from(value)
  return (chars[0] ?? '?').toUpperCase()
}

export function avatarColor(seed: string | null | undefined): string {
  const value = seed ?? ''
  let hash = 0
  for (let i = 0; i < value.length; i += 1) {
    hash = value.charCodeAt(i) + ((hash << 5) - hash)
  }
  return AVATAR_COLORS[Math.abs(hash) % AVATAR_COLORS.length] ?? '#0e7090'
}

export function otherMember(chat: Chat, userId: string | null | undefined): User | null {
  if (!userId) {
    return chat.member[0]?.user ?? null
  }
  return chat.member.find(member => member.user.id !== userId)?.user ?? null
}

export function chatTitle(
  chat: Chat,
  userId: string | null | undefined,
  untitledGroup: string,
): string {
  if (chat.type === 'Group') {
    const name = chat.name?.trim()
    return name || untitledGroup
  }
  return otherMember(chat, userId)?.username ?? untitledGroup
}

export function parseDate(iso: string): Date | null {
  const date = new Date(iso)
  return Number.isNaN(date.getTime()) ? null : date
}

export function dayKey(iso: string): string {
  const date = parseDate(iso)
  if (!date) {
    return ''
  }
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
}

function calendarDayDiff(date: Date, now = new Date()): number {
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const startOfThatDay = new Date(date.getFullYear(), date.getMonth(), date.getDate())
  return Math.round((startOfToday.getTime() - startOfThatDay.getTime()) / 86400000)
}

export function formatClockTime(iso: string, locale: string): string {
  const date = parseDate(iso)
  if (!date) {
    return ''
  }
  return date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit', hour12: false })
}

export function formatChatTime(iso: string, locale: string, yesterday: string): string {
  const date = parseDate(iso)
  if (!date) {
    return ''
  }

  const diffDays = calendarDayDiff(date)
  const time = formatClockTime(iso, locale)

  if (diffDays === 0) {
    return time
  }
  if (diffDays === 1) {
    return yesterday
  }
  if (diffDays > 1 && diffDays < 7) {
    return date.toLocaleDateString(locale, { weekday: 'short' })
  }
  return date.toLocaleDateString(locale, { month: '2-digit', day: '2-digit' })
}

export function formatDateDivider(
  iso: string,
  locale: string,
  today: string,
  yesterday: string,
): string {
  const date = parseDate(iso)
  if (!date) {
    return ''
  }

  const now = new Date()
  const diffDays = calendarDayDiff(date, now)

  if (diffDays === 0) {
    return today
  }
  if (diffDays === 1) {
    return yesterday
  }
  if (diffDays > 1 && diffDays < 7) {
    return date.toLocaleDateString(locale, { weekday: 'long' })
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString(locale, { month: 'long', day: 'numeric' })
  }
  return date.toLocaleDateString(locale, { year: 'numeric', month: 'long', day: 'numeric' })
}

export function formatDate(iso: string, locale: string): string {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) {
    return ''
  }
  return date.toLocaleDateString(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

export function shortId(id: string): string {
  return id.replace(/-/g, '').slice(0, 8)
}
