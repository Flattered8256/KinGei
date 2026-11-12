import type { ClientEvent } from '~/types/api'
import { setTyping as postTyping } from '~/utils/api/message'

const LOCAL_TTL_MS = 10_000
const REFRESH_MS = 4_000

export const useTypingStore = defineStore('typing', () => {
  const byChat = ref<Record<number, Record<string, true>>>({})
  const timers = new Map<string, ReturnType<typeof setTimeout>>()
  const live = new Set<number>()
  const lastTrueAt = new Map<number, number>()

  function typers(chatId: number, selfId?: string | null) {
    const ids = Object.keys(byChat.value[chatId] ?? {})
    if (!selfId) {
      return ids
    }
    return ids.filter(id => id !== selfId)
  }

  function isTyping(chatId: number, selfId?: string | null) {
    return typers(chatId, selfId).length > 0
  }

  function set(chatId: number, userId: string, typing: boolean) {
    const current = byChat.value[chatId] ?? {}
    const already = Boolean(current[userId])
    if (typing === already) {
      if (typing) {
        arm(chatId, userId)
      }
      return
    }
    if (typing) {
      byChat.value = {
        ...byChat.value,
        [chatId]: { ...current, [userId]: true },
      }
      arm(chatId, userId)
      return
    }
    disarm(chatId, userId)
    const { [userId]: _, ...rest } = current
    if (Object.keys(rest).length === 0) {
      const next = { ...byChat.value }
      delete next[chatId]
      byChat.value = next
      return
    }
    byChat.value = { ...byChat.value, [chatId]: rest }
  }

  function ingest(event: ClientEvent) {
    if (event.event === 'typing.changed') {
      set(event.data.chat_id, event.data.user_id, event.data.typing)
      return
    }
    if (event.event === 'message.created') {
      set(event.data.chat_id, event.data.sender_id, false)
      return
    }
    if (event.event === 'chat.deleted') {
      dropChat(event.data.chat_id)
      return
    }
    if (event.event === 'chat.member.removed') {
      set(event.data.chat_id, event.data.user_id, false)
    }
  }

  async function report(chatId: number | null, typing: boolean) {
    if (chatId == null) {
      return
    }
    if (typing) {
      const last = lastTrueAt.get(chatId) ?? 0
      if (live.has(chatId) && Date.now() - last < REFRESH_MS) {
        return
      }
      live.add(chatId)
      lastTrueAt.set(chatId, Date.now())
    }
    else {
      if (!live.has(chatId)) {
        return
      }
      live.delete(chatId)
      lastTrueAt.delete(chatId)
    }
    try {
      await postTyping(chatId, typing)
    }
    catch {
      // Typing is ephemeral; a failed ping must not toast or block composing.
      if (typing) {
        live.delete(chatId)
        lastTrueAt.delete(chatId)
      }
    }
  }

  function reset() {
    for (const timer of timers.values()) {
      clearTimeout(timer)
    }
    timers.clear()
    live.clear()
    lastTrueAt.clear()
    byChat.value = {}
  }

  function dropChat(chatId: number) {
    const users = byChat.value[chatId]
    if (users) {
      for (const userId of Object.keys(users)) {
        disarm(chatId, userId)
      }
    }
    if (chatId in byChat.value) {
      const next = { ...byChat.value }
      delete next[chatId]
      byChat.value = next
    }
    live.delete(chatId)
    lastTrueAt.delete(chatId)
  }

  function timerKey(chatId: number, userId: string) {
    return `${chatId}:${userId}`
  }

  function arm(chatId: number, userId: string) {
    const key = timerKey(chatId, userId)
    const prev = timers.get(key)
    if (prev) {
      clearTimeout(prev)
    }
    timers.set(key, setTimeout(() => {
      timers.delete(key)
      set(chatId, userId, false)
    }, LOCAL_TTL_MS))
  }

  function disarm(chatId: number, userId: string) {
    const key = timerKey(chatId, userId)
    const prev = timers.get(key)
    if (!prev) {
      return
    }
    clearTimeout(prev)
    timers.delete(key)
  }

  return {
    byChat,
    typers,
    isTyping,
    set,
    ingest,
    report,
    reset,
  }
})
