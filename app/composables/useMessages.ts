import type { ClientEvent, Message } from '~/types/api'
import { listMessages, sendMessage, updateReadCursor } from '~/utils/api/message'

const PAGE_SIZE = 50

export function useMessages() {
  const { user } = useAuth()
  const chats = useChats()

  const currentId = useState<number | null>('messages.currentId', () => null)
  const items = useState<Message[]>('messages.items', () => [])
  const loading = useState('messages.loading', () => false)
  const loadingMore = useState('messages.loadingMore', () => false)
  const hasMore = useState('messages.hasMore', () => true)
  const sending = useState('messages.sending', () => false)
  const error = useState<string | null>('messages.error', () => null)

  const myId = computed(() => user.value?.id ?? null)

  function resetThread() {
    items.value = []
    hasMore.value = true
    error.value = null
  }

  async function markRead(chatId: number, messageId?: number) {
    const last = messageId ?? items.value[items.value.length - 1]?.id
    if (!last) {
      return
    }
    try {
      await updateReadCursor(chatId, { last_read_message_id: last })
      chats.markReadLocal(chatId)
    }
    catch {
      // Read cursor is best-effort.
    }
  }

  async function open(chatId: number) {
    if (currentId.value === chatId && items.value.length > 0) {
      await markRead(chatId)
      return
    }
    currentId.value = chatId
    loading.value = true
    resetThread()
    try {
      const page = await listMessages(chatId, { limit: PAGE_SIZE })
      items.value = page
      hasMore.value = page.length >= PAGE_SIZE
      await markRead(chatId)
    }
    catch (err) {
      error.value = useApiI18n().fromError(err)
    }
    finally {
      loading.value = false
    }
  }

  async function loadOlder() {
    const chatId = currentId.value
    const first = items.value[0]
    if (!chatId || !first || loadingMore.value || !hasMore.value) {
      return
    }
    loadingMore.value = true
    try {
      const page = await listMessages(chatId, { before: first.id, limit: PAGE_SIZE })
      hasMore.value = page.length >= PAGE_SIZE
      items.value = [...page, ...items.value]
    }
    catch (err) {
      error.value = useApiI18n().fromError(err)
    }
    finally {
      loadingMore.value = false
    }
  }

  async function send(content: string) {
    const chatId = currentId.value
    const text = content.trim()
    if (!chatId || !text) {
      return
    }
    sending.value = true
    try {
      const created = await sendMessage(chatId, { content: text })
      if (!items.value.some(item => item.id === created.id)) {
        items.value = [...items.value, created]
      }
      await markRead(chatId, created.id)
    }
    finally {
      sending.value = false
    }
  }

  async function refreshCurrent() {
    if (currentId.value) {
      const id = currentId.value
      currentId.value = null
      await open(id)
    }
  }

  function ingest(event: ClientEvent) {
    if (event.event === 'message.created' && event.data.chat_id === currentId.value) {
      if (!items.value.some(item => item.id === event.data.id)) {
        items.value = [...items.value, event.data]
      }
      if (event.data.sender_id !== myId.value) {
        void markRead(event.data.chat_id, event.data.id)
      }
    }
  }

  function reset() {
    currentId.value = null
    items.value = []
    loading.value = false
    loadingMore.value = false
    hasMore.value = true
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
    loadOlder,
    send,
    refreshCurrent,
    ingest,
    reset,
  }
}
