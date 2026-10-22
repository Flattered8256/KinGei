import type { Chat, Friend, FriendRequest, User } from '~/types/api'

export function useUserDirectory() {
  const byId = useState<Record<string, User>>('users.byId', () => ({}))

  function remember(user: User | null | undefined) {
    if (!user) {
      return
    }
    byId.value[user.id] = user
  }

  function rememberMany(users: Array<User | null | undefined>) {
    for (const user of users) {
      remember(user)
    }
  }

  function rememberChat(chat: Chat) {
    rememberMany(chat.member.map(member => member.user))
  }

  function rememberFriend(friend: Friend) {
    remember({
      id: friend.friend_id,
      username: friend.username,
      email: friend.email,
      created_at: friend.created_at,
    })
  }

  function rememberRequest(request: FriendRequest) {
    rememberMany([request.sender, request.receiver])
  }

  function get(id: string): User | undefined {
    return byId.value[id]
  }

  function label(id: string, fallback: string): string {
    return byId.value[id]?.username ?? fallback
  }

  function reset() {
    byId.value = {}
  }

  return {
    byId,
    remember,
    rememberMany,
    rememberChat,
    rememberFriend,
    rememberRequest,
    get,
    label,
    reset,
  }
}
