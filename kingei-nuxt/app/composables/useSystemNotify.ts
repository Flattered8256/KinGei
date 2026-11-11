import type { Chat, FriendRequest, Message } from '~/types/api'
import { chatTitle, messagePreviewText } from '~/utils/display'

const STORAGE_KEY = 'kingei.notify'
const BODY_MAX = 120

type Notice = {
  tag: string
  title: string
  body: string
  icon: string
  href: string
}

const openByTag = new Map<string, Notification>()
let watchingPermission = false

export function useSystemNotify() {
  const { user } = useAuth()
  const { t } = useI18n()
  const chats = useChats()
  const users = useUserDirectory()

  const enabled = useState('notify.enabled', () => true)
  const permission = useState<NotificationPermission>('notify.permission', () => 'default')
  const asked = useState('notify.asked', () => false)

  const supported = computed(() =>
    import.meta.client && window.isSecureContext && typeof Notification === 'function',
  )
  const denied = computed(() => supported.value && permission.value === 'denied')
  const armed = computed(() =>
    enabled.value && supported.value && permission.value === 'granted',
  )

  function readStored(): boolean {
    if (!import.meta.client) {
      return true
    }
    const value = localStorage.getItem(STORAGE_KEY)
    if (value === 'off') {
      return false
    }
    return true
  }

  function persist(next: boolean) {
    enabled.value = next
    if (import.meta.client) {
      localStorage.setItem(STORAGE_KEY, next ? 'on' : 'off')
    }
  }

  function syncPermission() {
    if (!supported.value) {
      return
    }
    permission.value = Notification.permission
  }

  function watchPermission() {
    if (watchingPermission || !import.meta.client || !supported.value) {
      return
    }
    watchingPermission = true
    if (!navigator.permissions?.query) {
      return
    }
    void navigator.permissions.query({ name: 'notifications' }).then((status) => {
      status.addEventListener('change', syncPermission)
      syncPermission()
    }).catch(() => {
      // Safari may reject notifications permission query.
    })
  }

  function init() {
    if (!import.meta.client) {
      return
    }
    enabled.value = readStored()
    syncPermission()
    watchPermission()
  }

  init()

  async function requestPermission(): Promise<NotificationPermission> {
    if (!supported.value) {
      return 'denied'
    }
    try {
      const result = await Notification.requestPermission()
      permission.value = result
      return result
    }
    catch {
      syncPermission()
      return permission.value
    }
  }

  async function setArmed(next: boolean) {
    if (!next) {
      persist(false)
      return
    }
    if (!supported.value || permission.value === 'denied') {
      persist(false)
      return
    }
    const result = permission.value === 'granted'
      ? 'granted'
      : await requestPermission()
    persist(result === 'granted')
  }

  async function requestIfNeeded() {
    if (asked.value || !enabled.value || !supported.value) {
      return
    }
    if (permission.value !== 'default') {
      return
    }
    asked.value = true
    const result = await requestPermission()
    if (result === 'denied') {
      persist(false)
    }
  }

  function truncate(text: string) {
    const chars = Array.from(text)
    if (chars.length <= BODY_MAX) {
      return text
    }
    return `${chars.slice(0, BODY_MAX).join('')}…`
  }

  function previewOf(message: Message) {
    return messagePreviewText(message, {
      image: t('chat.previewImage'),
      video: t('chat.previewVideo'),
      file: t('chat.previewFile'),
      deleted: t('chat.messageDeleted'),
    })
  }

  function fileIcon(fileId: number | null | undefined) {
    if (fileId == null) {
      return `${location.origin}/favicon.ico`
    }
    return `${location.origin}/api/file/${fileId}/content`
  }

  function copyOf(message: Message) {
    const chat = chats.chats.value.find(item => item.id === message.chat_id)
    const senderName = users.label(message.sender_id, t('app.unknownUser'))
    const preview = previewOf(message)
    if (chat?.type === 'Group') {
      return {
        title: chatTitle(chat, user.value?.id, t('chat.untitledGroup')),
        body: truncate(t('notify.groupMessage', { name: senderName, text: preview })),
      }
    }
    return {
      title: chat ? chats.titleOf(chat) : senderName,
      body: truncate(preview),
    }
  }

  function closeAll() {
    for (const n of openByTag.values()) {
      n.close()
    }
    openByTag.clear()
  }

  function show(notice: Notice) {
    if (!armed.value || !import.meta.client) {
      return
    }
    if (document.hasFocus() || !notice.title) {
      return
    }
    const prev = openByTag.get(notice.tag)
    prev?.close()
    let n: Notification
    try {
      n = new Notification(notice.title, {
        body: notice.body,
        icon: notice.icon,
        tag: notice.tag,
      })
    }
    catch {
      // permission can be revoked between the armed check and the constructor
      return
    }
    openByTag.set(notice.tag, n)
    n.onclose = () => {
      if (openByTag.get(notice.tag) === n) {
        openByTag.delete(notice.tag)
      }
    }
    n.onclick = () => {
      n.close()
      window.focus()
      void navigateTo(notice.href)
    }
  }

  function onMessage(message: Message) {
    if (message.sender_id === user.value?.id || message.deleted) {
      return
    }
    const { title, body } = copyOf(message)
    const sender = users.get(message.sender_id)
    show({
      tag: `chat-${message.chat_id}`,
      title,
      body,
      icon: fileIcon(sender?.avatar),
      href: `/chat/${message.chat_id}`,
    })
  }

  function onFriendRequest(request: FriendRequest) {
    if (request.receiver.id !== user.value?.id) {
      return
    }
    show({
      tag: `friend-${request.id}`,
      title: request.sender.username,
      body: t('notify.friendRequest'),
      icon: fileIcon(request.sender.avatar),
      href: '/friends',
    })
  }

  function onFriendAccepted(request: FriendRequest) {
    const me = user.value?.id
    if (!me || (request.sender.id !== me && request.receiver.id !== me)) {
      return
    }
    const other = request.sender.id === me ? request.receiver : request.sender
    show({
      tag: `friend-${request.id}`,
      title: other.username,
      body: t('notify.nowFriends'),
      icon: fileIcon(other.avatar),
      href: `/friends/${other.id}`,
    })
  }

  function onGroupInvite(chat: Chat) {
    if (chat.type !== 'Group') {
      return
    }
    show({
      tag: `invite-${chat.id}`,
      title: chatTitle(chat, user.value?.id, t('chat.untitledGroup')),
      body: t('notify.groupInvite'),
      icon: fileIcon(chat.avatar),
      href: `/chat/${chat.id}`,
    })
  }

  return {
    supported,
    denied,
    armed,
    permission,
    setArmed,
    requestIfNeeded,
    onMessage,
    onFriendRequest,
    onFriendAccepted,
    onGroupInvite,
    closeAll,
  }
}
