import type { ClientEvent, Message, SendMessageRequest } from '~/types/api'
import { listMessages, sendMessage, updateReadCursor } from '~/utils/api/message'

const PAGE_SIZE = 50

type Thread = {
  items: Message[]
  hasMore: boolean
}

const emptyThread: Thread = { items: [], hasMore: true }

export function useMessages() {
  const { user } = useAuth()
  const apiI18n = useApiI18n()
  const toast = useToast()
  const chats = useChats()

  const currentId = useState<number | null>('messages.currentId', () => null)
  const threads = useState<Record<number, Thread>>('messages.threads', () => ({}))
  const drafts = useState<Record<number, string>>('messages.drafts', () => ({}))
  const loading = useState('messages.loading', () => false)
  const loadingMore = useState('messages.loadingMore', () => false)
  const sending = useState('messages.sending', () => false)
  const error = useState<string | null>('messages.error', () => null)

  const myId = computed(() => user.value?.id ?? null)
  const currentThread = computed(() => {
    const id = currentId.value
    return id == null ? emptyThread : threads.value[id] ?? emptyThread
  })
  const items = computed(() => currentThread.value.items)
  const hasMore = computed(() => currentThread.value.hasMore)

  function isCurrent(chatId: number) {
    return currentId.value === chatId
  }

  function setThread(chatId: number, next: Thread) {
    threads.value = { ...threads.value, [chatId]: next }
  }

  function dropThread(chatId: number) {
    if (!(chatId in threads.value) && !(chatId in drafts.value)) {
      return
    }
    const nextThreads = { ...threads.value }
    delete nextThreads[chatId]
    threads.value = nextThreads
    const nextDrafts = { ...drafts.value }
    delete nextDrafts[chatId]
    drafts.value = nextDrafts
  }

  function threadOf(chatId: number) {
    return threads.value[chatId]
  }

  function appendMessage(chatId: number, message: Message) {
    const thread = threadOf(chatId)
    if (!thread || thread.items.some(item => item.id === message.id)) {
      return
    }
    setThread(chatId, { ...thread, items: [...thread.items, message] })
  }

  function mergeLatest(chatId: number, page: Message[]) {
    const thread = threadOf(chatId)
    const prev = thread?.items ?? []
    if (prev.length === 0) {
      setThread(chatId, { items: page, hasMore: page.length >= PAGE_SIZE })
      return
    }
    const oldest = page[0]
    if (!oldest) {
      return
    }
    const overlap = prev.some(item => item.id >= oldest.id)
    if (overlap) {
      const kept = prev.filter(item => item.id < oldest.id)
      setThread(chatId, {
        items: [...kept, ...page],
        hasMore: kept.length > 0 ? (thread?.hasMore ?? true) : page.length >= PAGE_SIZE,
      })
      return
    }
    const last = prev[prev.length - 1]
    if (last && oldest.id > last.id) {
      setThread(chatId, { items: page, hasMore: true })
    }
  }

  async function markRead(chatId: number, messageId?: number) {
    if (!isCurrent(chatId)) {
      return
    }
    const last = messageId ?? threadOf(chatId)?.items.at(-1)?.id
    if (!last) {
      return
    }
    try {
      await updateReadCursor(chatId, { last_read_message_id: last })
      if (!isCurrent(chatId)) {
        return
      }
      chats.markReadLocal(chatId)
    }
    catch {
      // Read cursor is best-effort.
    }
  }

  function appendNewer(chatId: number, page: Message[]) {
    const thread = threadOf(chatId)
    if (!thread) {
      if (page.length > 0) {
        setThread(chatId, { items: page, hasMore: true })
      }
      return
    }
    const have = new Set(thread.items.map(item => item.id))
    const extra = page.filter(item => !have.has(item.id))
    if (extra.length === 0) {
      return
    }
    setThread(chatId, { ...thread, items: [...thread.items, ...extra] })
  }

  async function syncLatest(chatId: number) {
    const page = await listMessages(chatId, { limit: PAGE_SIZE })
    mergeLatest(chatId, page)
  }

  async function pullNewer(chatId: number) {
    const lastId = threadOf(chatId)?.items.at(-1)?.id
    if (lastId == null) {
      await syncLatest(chatId)
      return
    }
    const unread = chats.chats.value.find(item => item.id === chatId)?.unread_count ?? 0
    if (unread >= PAGE_SIZE) {
      await syncLatest(chatId)
      return
    }
    const batch = await listMessages(chatId, { after: lastId, limit: PAGE_SIZE })
    if (batch.length >= PAGE_SIZE) {
      await syncLatest(chatId)
      return
    }
    appendNewer(chatId, batch)
  }

  async function open(chatId: number) {
    const cached = threadOf(chatId)
    currentId.value = chatId
    loadingMore.value = false
    error.value = null

    if (cached) {
      loading.value = false
      try {
        await pullNewer(chatId)
      }
      catch {
        // Cached messages stay on screen.
      }
      if (isCurrent(chatId)) {
        await markRead(chatId)
      }
      return
    }

    loading.value = true
    try {
      const page = await listMessages(chatId, { limit: PAGE_SIZE })
      setThread(chatId, { items: page, hasMore: page.length >= PAGE_SIZE })
      if (!isCurrent(chatId)) {
        return
      }
      await markRead(chatId)
    }
    catch (err) {
      if (isCurrent(chatId)) {
        error.value = apiI18n.fromError(err)
        toast.error(err)
      }
    }
    finally {
      if (isCurrent(chatId)) {
        loading.value = false
      }
    }
  }

  function close(chatId?: number) {
    if (chatId != null && !isCurrent(chatId)) {
      return
    }
    currentId.value = null
    loading.value = false
    loadingMore.value = false
    sending.value = false
    error.value = null
  }

  async function loadOlder() {
    const chatId = currentId.value
    const thread = chatId != null ? threadOf(chatId) : undefined
    const first = thread?.items[0]
    if (!chatId || !thread || !first || loadingMore.value || !thread.hasMore) {
      return
    }
    loadingMore.value = true
    try {
      const page = await listMessages(chatId, { before: first.id, limit: PAGE_SIZE })
      const latest = threadOf(chatId)
      if (!latest) {
        return
      }
      const have = new Set(latest.items.map(item => item.id))
      const older = page.filter(item => !have.has(item.id))
      setThread(chatId, {
        items: [...older, ...latest.items],
        hasMore: page.length >= PAGE_SIZE,
      })
    }
    catch (err) {
      if (isCurrent(chatId)) {
        error.value = apiI18n.fromError(err)
        toast.error(err)
      }
    }
    finally {
      if (isCurrent(chatId)) {
        loadingMore.value = false
      }
    }
  }

  async function send(content: string, fileId?: number) {
    const chatId = currentId.value
    const text = content.trim()
    if (!chatId || (!text && fileId == null)) {
      return
    }
    sending.value = true
    try {
      const body: SendMessageRequest = {}
      if (text) {
        body.content = text
      }
      if (fileId != null) {
        body.file_id = fileId
      }
      const created = await sendMessage(chatId, body)
      appendMessage(chatId, created)
      await markRead(chatId, created.id)
    }
    finally {
      if (isCurrent(chatId)) {
        sending.value = false
      }
    }
  }

  async function refreshCurrent() {
    const id = currentId.value
    if (!id) {
      return
    }
    try {
      await pullNewer(id)
      await markRead(id)
    }
    catch {
      // Reconnect refresh is best-effort.
    }
  }

  function ingest(event: ClientEvent) {
    switch (event.event) {
      case 'message.created': {
        appendMessage(event.data.chat_id, event.data)
        if (event.data.sender_id !== myId.value) {
          void markRead(event.data.chat_id, event.data.id)
        }
        break
      }
      case 'chat.deleted':
        dropThread(event.data.chat_id)
        break
      case 'chat.member.removed':
        if (event.data.user_id === myId.value) {
          dropThread(event.data.chat_id)
        }
        break
      default:
        break
    }
  }

  function takeDraft(chatId: number | null) {
    if (chatId == null) {
      return ''
    }
    return drafts.value[chatId] ?? ''
  }

  function saveDraft(chatId: number | null, text: string) {
    if (chatId == null) {
      return
    }
    if (!text) {
      if (!(chatId in drafts.value)) {
        return
      }
      const next = { ...drafts.value }
      delete next[chatId]
      drafts.value = next
      return
    }
    drafts.value = { ...drafts.value, [chatId]: text }
  }

  function reset() {
    currentId.value = null
    threads.value = {}
    drafts.value = {}
    loading.value = false
    loadingMore.value = false
    sending.value = false
    error.value = null
  }

  return {
    currentId,
    items,
    loading,
    loadingMore,
    hasMore,
    sending,
    error,
    open,
    close,
    loadOlder,
    send,
    refreshCurrent,
    ingest,
    takeDraft,
    saveDraft,
    reset,
  }
}
