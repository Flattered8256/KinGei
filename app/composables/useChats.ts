import type { Chat, ClientEvent, Message } from '~/types/api'
import {
  createDirectChat,
  createGroupChat,
  getChat,
  inviteChatMember,
  listChats,
  removeChatMember,
  updateChat,
} from '~/utils/api/chat'
import { listMessages } from '~/utils/api/message'
import { chatTitle, otherMember } from '~/utils/display'

export function useChats() {
  const { user } = useAuth()
  const { t } = useI18n()
  const users = useUserDirectory()

  const chats = useState<Chat[]>('chats.list', () => [])
  const previews = useState<Record<number, Message>>('chats.previews', () => ({}))
  const loading = useState('chats.loading', () => false)
  const error = useState<string | null>('chats.error', () => null)

  const myId = computed(() => user.value?.id ?? null)
  const unreadTotal = computed(() =>
    chats.value.reduce((sum, chat) => sum + chat.unread_count, 0),
  )

  function sortChats(list: Chat[]) {
    return [...list].sort((a, b) => {
      const aTime = previews.value[a.id]?.created_at ?? a.created_at
      const bTime = previews.value[b.id]?.created_at ?? b.created_at
      return bTime.localeCompare(aTime)
    })
  }

  function replaceChat(next: Chat) {
    users.rememberChat(next)
    const index = chats.value.findIndex(item => item.id === next.id)
    if (index === -1) {
      chats.value = sortChats([next, ...chats.value])
      return
    }
    const copy = [...chats.value]
    copy[index] = next
    chats.value = sortChats(copy)
  }

  function removeChat(id: number) {
    chats.value = chats.value.filter(item => item.id !== id)
    const nextPreviews = { ...previews.value }
    delete nextPreviews[id]
    previews.value = nextPreviews
  }

  async function loadPreview(chatId: number) {
    try {
      const messages = await listMessages(chatId, { limit: 1 })
      const last = messages[messages.length - 1]
      if (last) {
        previews.value = { ...previews.value, [chatId]: last }
        chats.value = sortChats(chats.value)
      }
    }
    catch {
      // Preview is optional; the list still works without it.
    }
  }

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      const list = await listChats()
      for (const chat of list) {
        users.rememberChat(chat)
      }
      chats.value = sortChats(list)
      await Promise.all(list.map(chat => loadPreview(chat.id)))
    }
    catch (err) {
      error.value = useApiI18n().fromError(err)
      useToast().error(err)
    }
    finally {
      loading.value = false
    }
  }

  async function openDirect(friendId: string) {
    const existing = chats.value.find((chat) => {
      if (chat.type !== 'Direct') {
        return false
      }
      return chat.member.some(member => member.user.id === friendId)
    })
    if (existing) {
      return existing
    }
    const chat = await createDirectChat({ friend_id: friendId })
    replaceChat(chat)
    return chat
  }

  async function openGroup(name: string) {
    const chat = await createGroupChat({ name })
    replaceChat(chat)
    return chat
  }

  async function rename(id: number, name: string) {
    const chat = await updateChat(id, { name })
    replaceChat(chat)
    return chat
  }

  async function setAvatar(id: number, fileId: number) {
    const chat = await updateChat(id, { avatar: fileId })
    replaceChat(chat)
    return chat
  }

  async function invite(id: number, userId: string) {
    const chat = await inviteChatMember(id, { user_id: userId })
    replaceChat(chat)
    return chat
  }

  async function removeMember(id: number, userId: string) {
    await removeChatMember(id, userId)
    if (userId === myId.value) {
      removeChat(id)
      return
    }
    const chat = await getChat(id)
    replaceChat(chat)
  }

  function titleOf(chat: Chat) {
    return chatTitle(chat, myId.value, t('chat.untitledGroup'))
  }

  function peerOf(chat: Chat) {
    return otherMember(chat, myId.value)
  }

  function ingest(event: ClientEvent) {
    switch (event.event) {
      case 'chat.created':
      case 'chat.updated':
      case 'chat.member.added':
        replaceChat(event.data)
        break
      case 'chat.member.removed':
        if (event.data.user_id === myId.value) {
          removeChat(event.data.chat_id)
        }
        else {
          void getChat(event.data.chat_id).then(replaceChat).catch(() => {})
        }
        break
      case 'chat.deleted':
        removeChat(event.data.chat_id)
        break
      case 'message.created': {
        const message = event.data
        previews.value = { ...previews.value, [message.chat_id]: message }
        const chat = chats.value.find(item => item.id === message.chat_id)
        const viewing = import.meta.client && String(useRoute().params.id) === String(message.chat_id)
        if (chat && message.sender_id !== myId.value && !viewing) {
          replaceChat({ ...chat, unread_count: chat.unread_count + 1 })
        }
        else {
          chats.value = sortChats(chats.value)
        }
        break
      }
      case 'message.read': {
        const chat = chats.value.find(item => item.id === event.data.chat_id)
        if (!chat) {
          break
        }
        const preview = previews.value[chat.id]
        const hasNewerIncoming = Boolean(
          preview
          && preview.sender_id !== myId.value
          && preview.id > event.data.last_read_message_id,
        )
        replaceChat({
          ...chat,
          unread_count: event.data.user_id === myId.value && !hasNewerIncoming
            ? 0
            : chat.unread_count,
          member: chat.member.map(member =>
            member.user.id === event.data.user_id
              ? { ...member, last_read_message_id: event.data.last_read_message_id }
              : member,
          ),
        })
        break
      }
      default:
        break
    }
  }

  function markReadLocal(chatId: number) {
    const chat = chats.value.find(item => item.id === chatId)
    if (chat && chat.unread_count !== 0) {
      replaceChat({ ...chat, unread_count: 0 })
    }
  }

  function reset() {
    chats.value = []
    previews.value = {}
    loading.value = false
    error.value = null
  }

  return {
    chats,
    previews,
    unreadTotal,
    loading,
    error,
    refresh,
    openDirect,
    openGroup,
    rename,
    setAvatar,
    invite,
    removeMember,
    titleOf,
    peerOf,
    ingest,
    markReadLocal,
    replaceChat,
    removeChat,
    reset,
  }
}
