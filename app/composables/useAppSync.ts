import type { ClientEvent } from '~/types/api'

export function useAppSync() {
  const { isAuthenticated, user } = useAuth()
  const chats = useChats()
  const friends = useFriends()
  const messages = useMessages()
  const users = useUserDirectory()
  const router = useRouter()

  function ingest(event: ClientEvent) {
    chats.ingest(event)
    friends.ingest(event)
    messages.ingest(event)

    if (event.event === 'chat.deleted' && messages.currentId.value === event.data.chat_id) {
      void router.replace('/chat')
    }
    if (event.event === 'chat.member.removed'
      && event.data.user_id === user.value?.id
      && messages.currentId.value === event.data.chat_id) {
      void router.replace('/chat')
    }
  }

  async function bootstrap() {
    if (!isAuthenticated.value) {
      return
    }
    users.remember(user.value)
    await Promise.all([chats.refresh(), friends.refresh()])
  }

  function resetAll() {
    chats.reset()
    friends.reset()
    messages.reset()
    users.reset()
  }

  useEventStream({
    onEvent: ingest,
    onReconnect: () => {
      void bootstrap().then(() => messages.refreshCurrent())
    },
  })

  watch(isAuthenticated, (authenticated) => {
    if (authenticated) {
      void bootstrap()
      return
    }
    resetAll()
    void router.replace('/login')
  })

  onMounted(() => {
    if (isAuthenticated.value) {
      void bootstrap()
    }
  })

  return {
    bootstrap,
    resetAll,
  }
}
