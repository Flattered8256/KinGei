import type { Chat } from '~/types/api'

export function useTyping() {
  const store = useTypingStore()
  const { user } = useAuth()
  const users = useUserDirectory()
  const { t } = useI18n()

  const myId = computed(() => user.value?.id ?? null)

  function typers(chatId: number) {
    return store.typers(chatId, myId.value)
  }

  function isTyping(chatId: number) {
    return store.isTyping(chatId, myId.value)
  }

  function nameOf(chat: Chat, userId: string) {
    return chat.member.find(member => member.user.id === userId)?.user.username
      ?? users.get(userId)?.username
      ?? t('app.unknownUser')
  }

  function labelOf(chat: Chat) {
    const ids = typers(chat.id)
    if (ids.length === 0) {
      return ''
    }
    if (chat.type === 'Direct') {
      return t('chat.typingDirect')
    }
    const names = ids.map(id => nameOf(chat, id))
    if (names.length === 1) {
      return t('chat.typingOne', { name: names[0] })
    }
    if (names.length === 2) {
      return t('chat.typingTwo', { a: names[0], b: names[1] })
    }
    return t('chat.typingMany', { a: names[0], b: names[1], n: names.length })
  }

  return {
    typers,
    isTyping,
    labelOf,
    report: store.report,
    ingest: store.ingest,
    reset: store.reset,
  }
}
