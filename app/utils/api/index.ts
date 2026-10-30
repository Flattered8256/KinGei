export { ApiError, isApiError, isApiErrorCode } from './error'
export {
  clearSession,
  getAccessToken,
  getRefreshToken,
  getSession,
  onSessionChange,
  restoreSession,
  setSession,
  setUser,
} from './session'
export { apiFetch, apiFetchVoid, refreshSession } from './http'

export {
  changePassword,
  findUser,
  loginUser,
  refreshTokens,
  registerUser,
  updateUser,
} from './user'

export {
  acceptFriendRequest,
  cancelFriendRequest,
  deleteFriend,
  listFriends,
  listReceivedFriendRequests,
  listSentFriendRequests,
  rejectFriendRequest,
  sendFriendRequest,
} from './friend'

export {
  createDirectChat,
  createGroupChat,
  getChat,
  inviteChatMember,
  listChats,
  removeChatMember,
  updateChat,
} from './chat'

export {
  listMessages,
  sendMessage,
  updateReadCursor,
} from './message'

export { connectEventStream } from './event'
export type { EventStreamConnection, EventStreamHandlers } from './event'

export { getInstance } from './instance'

export {
  deleteAdminUser,
  listAdminUsers,
  updateInstance,
} from './admin'

export {
  completeFile,
  createFile,
  downloadFile,
  isAbortError,
  isImageType,
  isVideoType,
  putToPresign,
  uploadFile,
  uploadImage,
} from './file'
