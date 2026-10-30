export type ChatType = 'Direct' | 'Group'

export type FriendRequestStatus = 'Accept' | 'Pending' | 'Reject'

export const API_ERROR_CODES = [
  'UserAlreadyExists',
  'NotFound',
  'InvalidCredentials',
  'PasswordMismatch',
  'InvalidToken',
  'CannotAddSelf',
  'UserNotFound',
  'AlreadyFriends',
  'RequestAlreadyPending',
  'RequestNotFound',
  'NotRequestReceiver',
  'NotRequestSender',
  'RequestNotPending',
  'NotFriends',
  'AddFriendDisabled',
  'ChatNotFound',
  'NotMember',
  'NotOwner',
  'CannotModifyDirect',
  'AlreadyMember',
  'EmptyName',
  'CreateGroupDisabled',
  'EmptyContent',
  'MessageNotFound',
  'FileNotFound',
  'FileNameTooLong',
  'InvalidSize',
  'UploadIncomplete',
  'SizeMismatch',
  'FileNotReady',
  'InvalidImage',
  'InternalServerError',
] as const

export type ApiErrorCode = (typeof API_ERROR_CODES)[number]

export type UserRole = 'Admin' | 'Member'

export interface User {
  id: string
  username: string
  email: string
  created_at: string
  avatar: number | null
  role: UserRole
}

export interface Instance {
  allow_add_friend: boolean
  allow_create_group: boolean
}

export interface CreateUserRequest {
  username: string
  email: string
  password: string
}

export interface LoginRequest {
  email: string
  password: string
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  token_type: string
  user: User
}

export interface UpdateUserRequest {
  username?: string
  email?: string
  avatar?: number
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}

export interface RefreshTokenRequest {
  refresh_token: string
}

export interface RefreshTokenResponse {
  access_token: string
  refresh_token: string
  token_type: string
}

export interface Friend {
  user: User
  created_at: string
}

export interface SendFriendRequest {
  receiver_id: string
}

export interface FriendRequest {
  id: number
  sender: User
  receiver: User
  status: FriendRequestStatus
  created_at: string
  updated_at: string
}

export interface ChatMember {
  user: User
  last_read_message_id: number | null
}

export interface Chat {
  id: number
  type: ChatType
  name: string | null
  owner_id: string | null
  member: ChatMember[]
  created_at: string
  unread_count: number
  avatar: number | null
}

export interface CreateDirectRequest {
  friend_id: string
}

export interface CreateGroupRequest {
  name: string
}

export interface UpdateChatRequest {
  name?: string
  avatar?: number
}

export interface InviteMemberRequest {
  user_id: string
}

export type MediaKind = 'Image' | 'Video'

export interface MediaResponse {
  kind: MediaKind
  width: number
  height: number
  duration_ms: number | null
  thumbnail_id: number | null
}

export interface Message {
  id: number
  chat_id: number
  sender_id: string
  content: string
  file: FileResponse | null
  created_at: string
}

export interface SendMessageRequest {
  content?: string | null
  file_id?: number
}

export interface ListMessageQuery {
  before?: number
  after?: number
  limit?: number
}

export interface UpdateReadCursorRequest {
  last_read_message_id: number
}

export interface ReadCursor {
  last_read_message_id: number
  unread_count: number
}

export interface ReadReceiptEvent {
  chat_id: number
  user_id: string
  last_read_message_id: number
}

export interface ChatMemberRemovedEvent {
  chat_id: number
  user_id: string
}

export interface ChatDeletedEvent {
  chat_id: number
}

export interface FriendDeletedEvent {
  friend_id: string
}

export const CLIENT_EVENT_NAMES = [
  'message.created',
  'message.read',
  'chat.created',
  'chat.updated',
  'chat.member.added',
  'chat.member.removed',
  'chat.deleted',
  'friend.request.created',
  'friend.request.accepted',
  'friend.request.rejected',
  'friend.request.cancelled',
  'friend.deleted',
] as const

export type ClientEventName = (typeof CLIENT_EVENT_NAMES)[number]

export type ClientEvent =
  | { event: 'message.created'; data: Message }
  | { event: 'message.read'; data: ReadReceiptEvent }
  | { event: 'chat.created'; data: Chat }
  | { event: 'chat.updated'; data: Chat }
  | { event: 'chat.member.added'; data: Chat }
  | { event: 'chat.member.removed'; data: ChatMemberRemovedEvent }
  | { event: 'chat.deleted'; data: ChatDeletedEvent }
  | { event: 'friend.request.created'; data: FriendRequest }
  | { event: 'friend.request.accepted'; data: FriendRequest }
  | { event: 'friend.request.rejected'; data: FriendRequest }
  | { event: 'friend.request.cancelled'; data: FriendRequest }
  | { event: 'friend.deleted'; data: FriendDeletedEvent }

export interface CreateFileRequest {
  name: string
  content_type?: string
  size: number
}

export interface PresignUploadResponse {
  id: number
  name: string
  content_type: string
  size: number
  upload_url: string
  expires_at: string
}

export interface FileResponse {
  id: number
  owner_id: string
  name: string
  content_type: string
  size: number
  status: 'Pending' | 'Ready'
  media: MediaResponse | null
  created_at: string
}

export interface PresignDownloadResponse {
  id: number
  name: string
  content_type: string
  size: number
  download_url: string
  expires_at: string
}
