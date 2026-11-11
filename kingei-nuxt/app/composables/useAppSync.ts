import type { ClientEvent } from '~/types/api'

export function useAppReady() {
  return useState('app.ready', () => false)
}

export function useAppSync() {
  const { isAuthenticated, user } = useAuth()
  const chats = useChats()
  const friends = useFriends()
  const messages = useMessages()
  const users = useUserDirectory()
  const instance = useInstance()
  const presence = usePresence()
  const notify = useSystemNotify()
  const sseStatus = useSseStatus()
  const router = useRouter()
  const ready = useAppReady()

  function ingest(event: ClientEvent) {
    const unseenGroup = event.event === 'chat.member.added'
      && !chats.chats.value.some(item => item.id === event.data.id)

    chats.ingest(event)
    friends.ingest(event)
    messages.ingest(event)
    presence.ingest(event)

    if (event.event === 'message.created') {
      notify.onMessage(event.data)
    }
    if (event.event === 'friend.request.created') {
      notify.onFriendRequest(event.data)
    }
    if (event.event === 'friend.request.accepted') {
      notify.onFriendAccepted(event.data)
    }
    if (event.event === 'chat.member.added' && unseenGroup) {
      notify.onGroupInvite(event.data)
    }
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
    try {
      await Promise.all([
        chats.refresh(),
        friends.refresh(),
        instance.refresh(),
        presence.refresh(),
      ])
    }
    finally {
      if (isAuthenticated.value) {
        ready.value = true
      }
    }
  }

  function resetAll() {
    notify.closeAll()
    chats.reset()
    friends.reset()
    messages.reset()
    users.reset()
    presence.reset()
    ready.value = false
  }

  useEventStream({
    onEvent: ingest,
    onReconnect: () => {
      void bootstrap().then(() => messages.refreshCurrent())
    },
  })

  watch(isAuthenticated, (authenticated) => {
    if (authenticated) {
      void bootstrap().then(() => notify.requestIfNeeded())
      return
    }
    resetAll()
    void router.replace('/login')
  })

  watch(
    [sseStatus, user],
    () => {
      presence.markSelf()
    },
    { immediate: true },
  )

  onMounted(() => {
    if (isAuthenticated.value) {
      void bootstrap().then(() => notify.requestIfNeeded())
    }
  })

  return {
    bootstrap,
    resetAll,
    ready,
  }
}
