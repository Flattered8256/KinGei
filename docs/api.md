# KinGei HTTP API

与 `kingei-axum` 当前代码一致的接口说明。字段结构以 `src/serializer/*` 为准，本文记录代码里读不出来的语义与契约。

改接口时同一提交里更新本文：新增字段可以直接加，改名/删字段/改错误码属于破坏性变更，要在提交信息里写明。

## 基础约定

- 端口由 `PORT` 决定；本地开发时 Nuxt 代理默认指向 `http://127.0.0.1:8000`。
- 请求与响应都是 JSON，字段 snake_case，无统一包裹层：成功直接返回资源。
- 失败只返回 `{ "code": "PascalCaseCode" }`，配合 HTTP 状态码；客户端按 `code` 映射文案。
- 时间：RFC 3339 UTC，例如 `2026-11-08T09:00:00Z`。
- 主键：用户为 UUID 字符串，其余资源为 `i64`。

## 鉴权

| 项 | 约定 |
| --- | --- |
| Access Token | 15 分钟，`src/utils/auth.rs` 的 `ACCESS_TOKEN_DURATION_MINUTES` |
| Refresh Token | 7 天，同一文件的 `REFRESH_TOKEN_DURATION_DAYS` |
| 受保护路由 | `Authorization: Bearer <access>` |
| 缺失/非法 header | 401 `InvalidCredentials` |
| 用 refresh 当 access | 401 `InvalidToken` |
| Cookie 例外 | 仅 `GET /file/{id}/content`、`GET /file/{id}/download` 接受 `kingei.access_token` |

- `POST /user/login` 与 `POST /user/refresh` 除返回 token 外，还下发 `Set-Cookie: kingei.access_token=...; Path=/; Max-Age=900; HttpOnly; SameSite=Lax`，供图片预览与下载使用。
- `POST /user/logout` 清理 cookie，返回 `204`，不要求鉴权。
- 密码只存 bcrypt hash；任何响应都不出现 `password` / `password_hash`。

## 错误码

HTTP 状态码：`400` 参数或规则不合法、`401` 未登录或凭据错误、`403` 已登录无权、`404` 资源不存在、`409` 状态冲突、`413` 上传超出体积上限、`500` 基础设施失败（正文固定 `InternalServerError`，细节只进日志）。

| 领域 | 错误码 |
| --- | --- |
| user | `UserAlreadyExists` `NotFound` `InvalidCredentials` `PasswordMismatch` `InvalidToken` `NotAdmin` `CannotDeleteSelf` `CannotDeleteAdmin` |
| friend | `CannotAddSelf` `UserNotFound` `AlreadyFriends` `RequestAlreadyPending` `RequestNotFound` `NotRequestReceiver` `NotRequestSender` `RequestNotPending` `NotFriends` `AddFriendDisabled` |
| chat | `CannotAddSelf` `UserNotFound` `NotFriends` `ChatNotFound` `NotMember` `NotOwner` `CannotModifyDirect` `AlreadyMember` `EmptyName` `CreateGroupDisabled` |
| message | `EmptyContent` `ChatNotFound` `NotFriends` `MessageNotFound` `NotMember` `CannotDeleteMessage` `CannotDeleteOld` `MessageAlreadyDeleted` `ReplyTargetDeleted` |
| file | `EmptyName` `FileNameTooLong` `InvalidSize` `InvalidUpload` `FileTooLarge` `FileNotFound` `NotOwner` `FileNotReady` `InvalidImage` |
| invite | `InviteNotFound` |

## 端点

### 公共

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| GET | `/health` | 不鉴权、不碰对象存储 | `200` |
| POST | `/user/register` | 注册；`invite_only` 时需要 `invite_code` | `UserResponse` |
| POST | `/user/login` | 登录 | `LoginResponse` + Set-Cookie |
| POST | `/user/logout` | 退出，清 cookie | `204` |
| POST | `/user/refresh` | 用 refresh 换新 token | `RefreshTokenResponse` + Set-Cookie |
| GET | `/instance` | 站点开关（不含邀请码） | `InstanceResponse` |

### 用户（Bearer）

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| PATCH | `/user/update` | 改 `username` / `email` / `avatar`，省略不动 | `UserResponse` |
| PATCH | `/user/change_password` | `old_password` + `new_password` | `UserResponse` |
| GET | `/user/{email}` | 按邮箱查用户 | `UserResponse` |

### 好友（Bearer）

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| GET | `/friend` | 好友列表 | `[FriendResponse]` |
| DELETE | `/friend/{friend_id}` | 删除好友 | `204` |
| POST | `/friend/request` | 发起申请，`receiver_id` | `FriendRequestResponse` |
| GET | `/friend/request/received` | 收到的申请 | `[FriendRequestResponse]` |
| GET | `/friend/request/sent` | 发出的申请 | `[FriendRequestResponse]` |
| PATCH | `/friend/request/{id}/accept` | 同意 | `FriendRequestResponse` |
| PATCH | `/friend/request/{id}/reject` | 拒绝 | `FriendRequestResponse` |
| DELETE | `/friend/request/{id}` | 撤回 | `FriendRequestResponse` |

### 会话与消息（Bearer）

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| POST | `/chat/direct` | 建私聊，`friend_id`；已存在则返回已有的 | `ChatResponse` |
| POST | `/chat/group` | 建群，`name` | `ChatResponse` |
| GET | `/chat` | 当前用户的会话列表 | `[ChatResponse]` |
| GET | `/chat/{id}` | 会话详情 | `ChatResponse` |
| PATCH | `/chat/{id}` | 群主改 `name` / `avatar` | `ChatResponse` |
| POST | `/chat/{id}/member` | 拉人，`user_id` | `ChatResponse` |
| DELETE | `/chat/{id}/member/{user_id}` | 移除成员 | `204` |
| GET | `/chat/{id}/message` | 历史消息，见下方分页 | `[MessageResponse]` |
| POST | `/chat/{id}/message` | 发消息，`content` 可空、`file_id` 可选 | `MessageResponse` |
| POST | `/chat/{id}/message/{message_id}` | 回复该条 | `MessageResponse` |
| DELETE | `/chat/{id}/message/{message_id}` | 软删 | `MessageResponse` |
| PATCH | `/chat/{id}/read` | 更新已读游标，`last_read_message_id` | `ReadCursorResponse` |

### 事件（Bearer）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/event` | SSE 长连接，`Cache-Control: no-cache`、`X-Accel-Buffering: no`，每 15 秒 `ping` 保活 |

事件名与载荷（载荷与对应 REST 响应同形）：

| 事件 | 载荷 |
| --- | --- |
| `message.created` | `MessageResponse` |
| `message.deleted` | `MessageResponse` |
| `message.read` | `{ chat_id, user_id, last_read_message_id }` |
| `chat.created` / `chat.updated` / `chat.member.added` | `ChatResponse` |
| `chat.member.removed` | `{ chat_id, user_id }` |
| `chat.deleted` | `{ chat_id }` |
| `friend.request.created` / `.accepted` / `.rejected` / `.cancelled` | `FriendRequestResponse` |
| `friend.deleted` | `{ friend_id }` |

### 文件（Bearer；`/content`、`/download` 例外接受 cookie）

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| POST | `/file` | 上传，`multipart/form-data`：`file` 必需，`name` 可选 | `FileResponse`（直接 `Ready`） |
| GET | `/file/{id}/download` | 流式下载，`Content-Disposition: attachment`，支持 `Range` | 二进制 `200` / `206` |
| GET | `/file/{id}/content` | 流式预览，支持 `Range` | 二进制 `200` / `206` |

### 管理（Bearer + Admin）

| 方法 | 路径 | 说明 | 响应 |
| --- | --- | --- | --- |
| GET | `/admin/user` | 用户列表 | `[UserResponse]` |
| DELETE | `/admin/user/{id}` | 删用户 | `204` |
| PATCH | `/admin/instance` | 改开关，省略不动 | `InstanceResponse` |
| GET | `/admin/invite` | 取邀请码，未生成 `404 InviteNotFound` | `{ code }` |
| POST | `/admin/invite` | 生成或替换 8 位码，无请求体 | `{ code }` |

## 行为要点

### 会话

- `ChatResponse.type` 对外字段名是 `type`（Rust 里叫 `chat_type`），值为 `Direct` / `Group`。
- `ChatResponse.avatar` 是 file id 或 `null`；私聊恒为 `null`。
- `avatar` 只有群主能改；`Direct` 会话改属性一律 `400 CannotModifyDirect`。
- `member` 内嵌完整 `UserResponse` + 该成员自己的 `last_read_message_id`。

### 消息

- 分页：`limit` 默认 50、上限 100（1..=100 会被夹取）；`before` 取更早的，`after` 取新增的，都不传则取最近一页。**返回数组按 id 升序**。
- 发送：`content` 为空且没有 `file_id` → `400 EmptyContent`。
- 回复：`reply_to` 只嵌一层，内层 `reply_to` 恒为 `null`；目标不在本会话 → `404 MessageNotFound`，已删除 → `409 ReplyTargetDeleted`。
- 软删：`deleted = true`，同时 `content` 置空、`file` 与 `reply_to` 为 `null`；重复删除 → `409 MessageAlreadyDeleted`；非本人且非管理员 → `403 CannotDeleteMessage`。
- 删除时限：仅当实例开关 `limit_message_delete = true` 时生效，超过 120 秒 → `400 CannotDeleteOld`。

### 文件与对象存储

- 存储后端：`STORAGE_DRIVER=local`（默认）时文件写在 `STORAGE_LOCAL_DIR` 下，`object_key` 即相对路径；`s3` 时写入 S3 兼容对象存储。两种后端统一由 `object_store` 读写，接口与响应不暴露后端差异。
- 上传一步：`POST /file` 以 `multipart/form-data` 提交，`file` 字段是文件本体，可选的 `name` 文本字段优先于 part 的文件名（避免 multipart 文件名编码差异）。服务端把请求体流式写到临时文件，累计字节超过 `MAX_UPLOAD_SIZE_MB` → `413 FileTooLarge`；非 multipart、缺 `file` 字段或多个 `file` 字段 → `400 InvalidUpload`。客户端应先用 `/instance` 的 `max_upload_size` 校验，避免在流式上传中被 413 中途断开连接。
- 后端中转：同一个临时文件先写入存储后端（local 落盘 / S3 multipart，object key 为 `{owner_id}/{uuid}` 且不进 JSON），再本地 `ffprobe`/`ffmpeg` 出缩略图，最后入库并直接返回 `Ready`，不再从存储回拉。落库失败会删掉已上传文件。
- 媒体探测：`image/jpeg` `image/png` `image/webp` `image/gif`（GIF 按 Image、忽略时长）、`video/mp4` `video/quicktime` `video/webm`。`media` 与 `file` 1:1，只有存在预览才插入；`{ kind, width, height, duration_ms, thumbnail_id }`，宽高是显示尺寸（90°/270° 已对调），缩略图长边 320px。
- 探测失败不阻断 `Ready`：没有视频流则不插行、`media: null`；探测成功但出图失败仍插行、`thumbnail_id: null`。
- `download`：任意登录用户，仅 `Ready`；由后端流式转发，`Content-Disposition: attachment`（ASCII `filename` + RFC 5987 `filename*`），支持 `Range`。可直接用同源链接导航，鉴权走 cookie。
- `content`：流式转发，不读进内存；`Content-Type` 取库中值，`Content-Disposition: inline`，`Accept-Ranges: bytes`，`Cache-Control: private, max-age=86400`；带 `Range` 且对象支持则 `206` + `Content-Range`。local 与 S3 的 Range 语义一致，由 `object_store` 处理。
- `Content-Type` 始终以数据库为准；local 后端不在文件系统上保存对象属性（`put_opts` 不接受 attributes）。
- 当图片用（用户/群头像）走 `service::file::require_image`：必须属于当前用户、`Ready`、MIME 在图片白名单内。失败 → `404 FileNotFound` / `403 NotOwner` / `400 FileNotReady` / `400 InvalidImage`。
- 当附件用走 `service::file::require_ready_file`：属于发送者且 `Ready`，不限 MIME；失败不创建消息。

### 实例与邀请码

- `InstanceResponse`：`allow_add_friend`、`allow_create_group`、`initializing`、`invite_only`、`limit_message_delete`、`max_upload_size`（字节）。
- `initializing = true` 表示还没有管理员：第一个注册的用户在写 `user` 的同一事务里成为 Admin 并置 `false`，此时免邀请码。
- `allow_add_friend = false` 时成员发好友申请 → `403 AddFriendDisabled`；`allow_create_group = false` 时建群 → `403 CreateGroupDisabled`。Admin 不受这两个开关限制，已有好友/群/消息不受影响。
- `invite_only = true` 且非 `initializing` 时，注册必须带正确 `invite_code`（trim + 大写后匹配），缺码/错码 → `404 InviteNotFound`。
- 邀请码 8 位，字母表 `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`，全站唯一一个，不预生成；开启邀请制不会自动生成，需 `POST /admin/invite`。

## 嵌套结构

- `UserResponse`：`{ id, username, email, created_at, avatar, role }`，`avatar` 是 file id 或 `null`，不含下载地址。
- `FriendResponse`：`{ user: UserResponse, created_at }`，不摊平好友字段。
- `FileResponse`：`{ id, owner_id, name, content_type, size, status, media, created_at }`，`status` 为 `Pending` / `Ready`。
- `MessageResponse`：`{ id, chat_id, sender_id, content, file, created_at, deleted, reply_to }`。
- `LoginResponse`：`{ access_token, refresh_token, token_type, user }`；`RefreshTokenResponse` 少一个 `user`。

## 环境变量

| 变量 | 必填 | 说明 |
| --- | --- | --- |
| `DATABASE_URL` | 是 | `sqlite:...` 或 `postgres://...`，须与编译 feature 一致；SQLite 自动建库文件 |
| `PORT` | 否 | 监听端口 |
| `JWT_SECRET` | 是 | 本机必须非空；Docker 中可空，由入口脚本生成到 `JWT_SECRET_FILE` |
| `JWT_SECRET_FILE` | 否 | 默认 `/kingei-data/jwt_secret`，仅 Docker 入口脚本读取 |
| `RUST_LOG` | 否 | tracing 过滤 |
| `STORAGE_DRIVER` | 否 | `local`（默认）/ `s3`；决定文件读写后端 |
| `STORAGE_LOCAL_DIR` | 否 | local 的存储根目录，默认 `./KinGei-data/files`；相对路径按进程工作目录解析 |
| `S3_ENDPOINT` / `S3_ACCESS_KEY` / `S3_SECRET_KEY` / `S3_BUCKET` | `s3` 时是 | S3 兼容对象存储（本机 Garage） |
| `S3_REGION` | 否 | 默认 `us-east-1` |
| `S3_PATH_STYLE` | 否 | 默认 `true` |
| `MAX_UPLOAD_SIZE_MB` | 否 | 单文件上传上限，默认 `100`；超限返回 `413 FileTooLarge` |
| `FFPROBE_PATH` / `FFMPEG_PATH` | 否 | 默认 `ffprobe` / `ffmpeg`，启动时校验 |

`.env` 不入库。启动时找不到 ffprobe/ffmpeg 直接退出并给中文提示。

`s3` 模式要求 bucket 已存在：`object_store` 没有建桶 API，启动时用 `ListObjectsV2` 校验（凭据需要 `ListBucket` 权限）；Garage compose 会自动建桶，外部 S3/MinIO 需先手动创建。`S3_MANAGE_BUCKET` 已移除。
