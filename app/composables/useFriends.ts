import type { ClientEvent, Friend, FriendRequest } from '~/types/api'
import {
  acceptFriendRequest,
  cancelFriendRequest,
  deleteFriend,
  listFriends,
  listReceivedFriendRequests,
  listSentFriendRequests,
  rejectFriendRequest,
  sendFriendRequest,
} from '~/utils/api/friend'
import { findUser } from '~/utils/api/user'

export function useFriends() {
  const { user } = useAuth()
  const users = useUserDirectory()

  const friends = useState<Friend[]>('friends.list', () => [])
  const incoming = useState<FriendRequest[]>('friends.incoming', () => [])
  const outgoing = useState<FriendRequest[]>('friends.outgoing', () => [])
  const loading = useState('friends.loading', () => false)
  const error = useState<string | null>('friends.error', () => null)

  const myId = computed(() => user.value?.id ?? null)
  const incomingCount = computed(() => incoming.value.length)

  function pendingOnly(list: FriendRequest[]) {
    return list.filter(item => item.status === 'Pending')
  }

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      const [friendList, received, sent] = await Promise.all([
        listFriends(),
        listReceivedFriendRequests(),
        listSentFriendRequests(),
      ])
      for (const friend of friendList) {
        users.rememberFriend(friend)
      }
      for (const request of received) {
        users.rememberRequest(request)
      }
      for (const request of sent) {
        users.rememberRequest(request)
      }
      friends.value = friendList
      incoming.value = pendingOnly(received)
      outgoing.value = pendingOnly(sent)
    }
    catch (err) {
      error.value = useApiI18n().fromError(err)
      useToast().error(err)
    }
    finally {
      loading.value = false
    }
  }

  async function lookup(email: string) {
    const found = await findUser(email)
    users.remember(found)
    return found
  }

  async function request(receiverId: string) {
    const created = await sendFriendRequest({ receiver_id: receiverId })
    users.rememberRequest(created)
    if (!outgoing.value.some(item => item.id === created.id)) {
      outgoing.value = [created, ...outgoing.value]
    }
    return created
  }

  async function accept(id: number) {
    await acceptFriendRequest(id)
    incoming.value = incoming.value.filter(item => item.id !== id)
    await refresh()
  }

  async function reject(id: number) {
    await rejectFriendRequest(id)
    incoming.value = incoming.value.filter(item => item.id !== id)
  }

  async function cancel(id: number) {
    await cancelFriendRequest(id)
    outgoing.value = outgoing.value.filter(item => item.id !== id)
  }

  async function remove(friendId: string) {
    await deleteFriend(friendId)
    friends.value = friends.value.filter(item => item.user.id !== friendId)
  }

  function friendById(id: string) {
    return friends.value.find(item => item.user.id === id) ?? null
  }

  function ingest(event: ClientEvent) {
    switch (event.event) {
      case 'friend.request.created':
        users.rememberRequest(event.data)
        if (event.data.receiver.id === myId.value) {
          incoming.value = [event.data, ...incoming.value.filter(item => item.id !== event.data.id)]
        }
        else if (event.data.sender.id === myId.value) {
          outgoing.value = [event.data, ...outgoing.value.filter(item => item.id !== event.data.id)]
        }
        break
      case 'friend.request.accepted':
        users.rememberRequest(event.data)
        incoming.value = incoming.value.filter(item => item.id !== event.data.id)
        outgoing.value = outgoing.value.filter(item => item.id !== event.data.id)
        void refresh()
        break
      case 'friend.request.rejected':
      case 'friend.request.cancelled':
        users.rememberRequest(event.data)
        incoming.value = incoming.value.filter(item => item.id !== event.data.id)
        outgoing.value = outgoing.value.filter(item => item.id !== event.data.id)
        break
      case 'friend.deleted':
        friends.value = friends.value.filter(item => item.user.id !== event.data.friend_id)
        break
      default:
        break
    }
  }

  function reset() {
    friends.value = []
    incoming.value = []
    outgoing.value = []
    loading.value = false
    error.value = null
  }

  return {
    friends,
    incoming,
    incomingCount,
    outgoing,
    loading,
    error,
    refresh,
    lookup,
    request,
    accept,
    reject,
    cancel,
    remove,
    friendById,
    ingest,
    reset,
  }
}
