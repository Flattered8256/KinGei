# Chat 项目风格规范

本文档描述本仓库的代码组织、命名、分层职责与提交约定。新增代码以本文为准；与旧代码不一致时，优先对齐 `friend` / `chat` 模块的写法，而不是早期 `user` 模块里的临时习惯。

原则：

- 结构先于技巧。业务按领域切开，每一层只做该层的事。
- 对外契约稳定。JSON 字段名、错误码、HTTP 状态码一旦对外暴露，不要随意改名。
- 内部错误不泄漏。数据库、JWT、bcrypt 的细节只写进日志，不返回给客户端。
- 少注释、写清楚。注释只解释“为什么”，不复述代码在做什么。

---

## 1. 技术栈与目录

| 项 | 约定 |
| --- | --- |
| 语言 | Rust，`edition = "2024"` |
| Web | Axum 0.8 |
| 运行时 | Tokio（`macros` + `rt-multi-thread`） |
| 数据库 | SQLite + sqlx 0.9 |
| 序列化 | serde / serde_json |
| 错误 | thiserror + 各领域 `IntoResponse` |
| 鉴权 | JWT（access + refresh）+ bcrypt |
| 日志 | tracing |

```
src/
  main.rs              启动、读环境变量、跑迁移、绑定端口
  state.rs             AppState：组装 repository / service
  database.rs          连接池
  router.rs            顶层路由组装
  router/<domain>.rs   某领域的路径与 HTTP 方法
  handler/<domain>.rs  提取参数，调用 service，包装响应
  service/<domain>.rs  业务规则
  repository/<domain>.rs  SQL 与事务
  model/<domain>.rs    数据库实体 / 枚举
  serializer/<domain>.rs  请求体、响应体
  error/<domain>.rs    领域错误与 HTTP 映射
  middleware/          中间件（鉴权等）
  utils/               与 HTTP 无关的工具（JWT 等）
  storage.rs           Garage S3 客户端（path-style、预签名）
migrations/            sqlx 迁移脚本
```

分层模块一律采用 **门面文件 + 子目录**：

```rust
// src/service.rs
pub mod user;
pub mod friend;
pub mod chat;
```

不要写成 `src/service/mod.rs`。领域顺序固定为 `user`、`friend`、`chat`、`message`、`file`，新增领域按依赖关系追加在后面。

---

## 2. 分层职责

依赖只能向下，禁止反向或跨层跳过：

```
router → handler → service → repository
                       ↘ model
handler / service → serializer
handler / service → error
repository → model
middleware → utils
```

禁止：

- `repository` 引用 `service`、`handler`、`serializer`、`AppError`
- `handler` 直接写 SQL 或调用 `repository`
- `model` 依赖 Axum / JSON 请求类型
- `serializer` 依赖 `repository` 或 `service`

### 2.1 Router

- 只负责路径、HTTP 方法、是否挂鉴权中间件。
- 公开接口用 `public_routes()`，需要登录的用 `protected_routes()`。
- 领域路径在 `src/router.rs` 里 `nest`，前缀用单数资源名：`/user`、`/friend`、`/chat`。
- `protected_routes()` 统一 `.layer(middleware::from_fn(auth::auth_middleware))`。

```rust
pub fn protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(friend::list_friends))
        .route("/{friend_id}", delete(friend::delete_friend))
        .route("/request", post(friend::send_request))
        .layer(middleware::from_fn(auth::auth_middleware))
}
```

同一资源的多个方法可以链式注册：`.route("/{id}", get(chat::get).patch(chat::update))`。

### 2.2 Handler

Handler 必须薄：

1. 从 `State` / `Extension` / `Path` / `Json` 取值
2. 调用对应 `service` 方法
3. 成功则 `Ok(Json(...))` 或 `Ok(StatusCode::NO_CONTENT)`

不要在 handler 里做校验、查库、拼装领域对象。

参数命名：

| 提取器 | 变量名 |
| --- | --- |
| `State<Arc<AppState>>` | `state`（不要叫 `service`） |
| `Extension<Uuid>` | `user_id`（当前登录用户） |
| `Json<T>` | `req` |
| `Path` 单值 | 与路径参数同名：`id`、`email`、`friend_id` |

```rust
pub async fn accept(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<i64>,
) -> Result<Json<FriendRequestResponse>, AppError> {
    let result = state.friend_service.accept(user_id, id).await?;
    Ok(Json(result))
}
```

无内容删除返回 `204`：

```rust
pub async fn delete_friend(...) -> Result<StatusCode, AppError> {
    state.friend_service.delete_friend(user_id, friend_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

### 2.3 Service

- 持有 `Arc<...Repository>`，在 `AppState::new` 里注入。
- 入参用 serializer 请求类型 + 当前用户 id，出参用 serializer 响应类型。
- 返回 `Result<T, AppError>`。
- 负责：权限、状态机、唯一性、是否好友、空名称等业务规则。
- 仓储返回 `None` 时用 `ok_or(AppError::<Domain>(<Domain>Error::...))` 转成领域错误。
- 私有辅助方法放在 `impl` 底部，如 `require_member`、`to_response`。

```rust
if self.friend_repo.find(user_id, friend_id).await?.is_none() {
    return Err(AppError::Chat(ChatError::NotFriends));
}
```

不要在 service 里拼 SQL。跨表写入如果必须原子，把事务放到 **同一个 repository 方法** 里（见 `accept_and_create_friend`）。

### 2.4 Repository

- 只接收 `SqlitePool`（或在方法内 `begin` 事务）。
- 返回 `Result<T, sqlx::Error>`，不要返回 `AppError`。
- 方法名用资源语义：`create` / `find` / `find_by_*` / `list` / `list_*` / `update_*` / `delete`。
- 查不到用 `fetch_optional` 返回 `Option`，不要在仓储层决定 HTTP 语义。
- 列表不要在循环里 `find_by_id`，用 `find_by_ids` 一次查出；多会话成员用 `list_members_by_chat_ids`，未读用 `unread_counts_for_*`；消息附件用 `find_by_ids` + `find_media_by_file_ids`。
- 多步写入用事务：

```rust
let mut tx = self.pool.begin().await?;
// ... 多次 query，executor 用 &mut *tx
tx.commit().await?;
```

动态 SET 字段用 `QueryBuilder`，不要手写字符串拼接用户输入。

### 2.5 Model

- 对应一张表（或表上的枚举列）。
- `#[derive(Debug, FromRow)]`。需要进 JSON 的枚举额外 `Serialize`。
- 字段与列名一致，snake_case。
- **不要**把 `password_hash` 放到响应类型里；model 可以有，serializer 必须去掉。

### 2.6 Serializer

- 请求：`#[derive(Debug, Deserialize)]`，类型名 `*Request`。
- 响应：`#[derive(Debug, Serialize)]`，类型名 `*Response`。
- 用 `From<Model>` 做简单映射；需要额外数据时用关联函数，例如 `ChatResponse::from(chat, member)`。
- JSON 字段默认 snake_case。Rust 字段与 JSON 冲突时用 `#[serde(rename = "...")]`，例如聊天类型字段对外叫 `type`。

### 2.7 Error

每个领域一个枚举，统一包进 `AppError`：

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    User(#[from] UserError),
    #[error("{0}")]
    Friend(#[from] FriendError),
    #[error("{0}")]
    Chat(#[from] ChatError),
    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
}
```

领域错误：

- `#[error("PascalCaseCode")]` 的文案 **等于** 变体名，也等于返回给客户端的 `code`。
- 实现 `IntoResponse`，body 只有 `{ "code": "..." }`。
- `Database` / `Bcrypt` / `JWT` 记 `tracing::error!`，对客户端一律 `500` + `InternalServerError`。

HTTP 状态码习惯：

| 场景 | 状态码 | 示例 code |
| --- | --- | --- |
| 参数/规则不合法 | 400 | `CannotAddSelf`、`EmptyName`、`CannotModifyDirect`、`InvalidImage` |
| 未登录、密码错、token 无效 | 401 | `InvalidCredentials`、`InvalidToken`、`PasswordMismatch` |
| 已登录但无权 | 403 | `NotOwner`、`NotMember`、`NotRequestReceiver` |
| 资源不存在 | 404 | `NotFound`、`ChatNotFound`、`NotFriends` |
| 与当前状态冲突 | 409 | `UserAlreadyExists`、`AlreadyFriends`、`AlreadyMember` |
| 基础设施失败 | 500 | `InternalServerError` |

---

## 3. 新增一个业务领域

以领域名 `foo` 为例，必须同时补齐：

1. `src/model/foo.rs` + `src/model.rs` 导出
2. `src/serializer/foo.rs` + 导出
3. `src/error/foo.rs` + 导出，并在 `AppError` 增加变体和 `IntoResponse` 分支
4. `src/repository/foo.rs` + 导出
5. `src/service/foo.rs` + 导出
6. `src/handler/foo.rs` + 导出
7. `src/router/foo.rs` + 在 `create_router` 里 `nest`
8. `AppState` 增加对应 service，并在 `new` 里注入 repository

不要只加表或只加 handler。

---

## 4. 命名

### 4.1 Rust

| 种类 | 风格 | 示例 |
| --- | --- | --- |
| 文件 / 模块 | snake_case，单数领域名 | `friend.rs`、`chat.rs` |
| 类型 | PascalCase | `UserService`、`CreateGroupRequest` |
| 函数 / 字段 / 变量 | snake_case | `list_received`、`owner_id` |
| 常量 | SCREAMING_SNAKE_CASE | `ACCESS_TOKEN_DURATION_MINUTES` |
| 错误枚举 | `<Domain>Error` | `ChatError` |
| 仓储 | `<Resource>Repository` | `FriendRequestRepository` |
| 更新用内部结构 | `Update<Resource>` | `UpdateUser`（放 repository，不进 JSON） |

### 4.2 数据库

- 表名 **单数**：`user`、`friend`、`chat`、`chat_member`、`friend_request`、`file`。
- 列名 snake_case，与 model 字段相同。
- 用户主键：`TEXT` 存 UUID。
- 其余资源主键：`INTEGER PRIMARY KEY AUTOINCREMENT`，Rust 侧 `i64`。
- 时间：`TEXT NOT NULL`，Rust 侧 `DateTime<Utc>`。
- 枚举列：`TEXT`，sqlx `#[sqlx(rename_all = "PascalCase")]`，值为 `Pending` / `Direct` 这种 PascalCase。
- 索引：`idx_<表>_<列>`，例如 `idx_friend_user_id`。
- 迁移文件：`migrations/YYYYMMDDHHMMSS_动词_名词.sql`，例如 `20260902030000_create_chats.sql`。只追加，不改已经提交的迁移。

### 4.3 HTTP
- restful风格：成功直接返回资源，失败则放回code字段，错误代码用简短的英文。
- 顶层前缀：`/user`、`/friend`、`/chat`、`/event`、`/file`。
- 路径用名词，动作用 HTTP 方法，不用 `doAccept` 这种动词当资源。
- 子资源可以出现动作后缀：`/request/{id}/accept`、`/request/{id}/reject`。
- 路径参数：`{id}`、`{email}`、`{friend_id}`、`{user_id}`，与 handler 的 `Path` 同名。
- JSON 请求/响应字段 snake_case，例外用 `serde(rename)` 显式写出。

---

## 5. Rust 代码格式

提交前执行：

```bash
cargo fmt
cargo clippy
cargo build
```

具体格式：

- 4 空格缩进，不用 Tab。
- 行宽按 rustfmt 默认（100）。
- 逗号后有空格：`bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)`。
- 多行参数、结构体、match 臂使用尾逗号。
- 不要在行尾留空白，不要用 `// filename.rs` 这种文件头注释。
- `use` 分组：标准库 → 第三方 → `crate::`，组之间空一行。同一组内按模块路径字母序。
- 优先 `crate::foo::bar`，不要 `super::` 跨多层。
- 异步函数：仓储和服务用 `&self`，不要把 `SqlitePool` 传来传去。
- 可空查询用 `if let Some(...)` / `let Some(x) = ... else { continue; }`，不要 `unwrap`。
- 启动期配置缺失可以用 `expect`，且信息用中文：`expect("缺少DATABASE_URL,请检查项目根目录下的.env文件")`。
- 请求路径上的失败用 `Result` + `?`，禁止 `unwrap`。

导入示例：

```rust
use std::sync::Arc;

use axum::{extract::State, Extension, Json};
use uuid::Uuid;

use crate::error::AppError;
use crate::serializer::chat::ChatResponse;
use crate::state::AppState;
```

---

## 6. SQL 写法

- 使用 `sqlx::query` / `query_as`，SQL 放在 `r#"` 原始字符串里。
- 占位符用 `?`，按 `.bind(...)` 顺序绑定。
- `INSERT` / `UPDATE` / `DELETE` 需要返回行时写 `RETURNING` 全列，列顺序与 `FromRow` 一致。
- 关键字大写，表名/列名小写：

```sql
SELECT id, user_id, friend_id, created_at
FROM friend
WHERE user_id = ? AND friend_id = ?
```

- 需要原子性的多条语句放进同一个 repository 方法的事务，不要在 service 里“连续 await 两次写入”冒充事务。
- 不要 `SELECT *`。

---

## 7. 鉴权与配置

- 环境变量：`DATABASE_URL`、`PORT`、`JWT_SECRET`、`RUST_LOG`、`GARAGE_ENDPOINT`、`GARAGE_ACCESS_KEY`、`GARAGE_SECRET_KEY`、`GARAGE_BUCKET`、`GARAGE_REGION`（缺省 `us-east-1`）、可选 `GARAGE_PUBLIC_ENDPOINT`（缺省等于 `GARAGE_ENDPOINT`，只用于预签名 URL）、可选 `FFPROBE_PATH`（缺省 `ffprobe`）、可选 `FFMPEG_PATH`（缺省 `ffmpeg`）。`.env` 不入库。启动时跑 `ffprobe -version` 和 `ffmpeg -version`，找不到则中文 `expect` 退出。
- Garage 为本机 path-style S3（本地 `http://127.0.0.1:3900`）。`Storage::connect` 启动时 `ListBuckets`，没有目标 bucket 则 `CreateBucket`；连不上则中止启动。随后 `PutBucketCors`：Methods `PUT` `GET` `HEAD`，Origins/Headers `*`，Expose `ETag` `Content-Length` `Content-Range`。
- `GET /health` 不鉴权、不碰 Garage，固定 200。
- SQLite 连接使用 `create_if_missing(true)`，库文件不存在则创建。
- `POST /file`（JWT）申请 PutObject 预签名 URL：请求 `name`、`content_type`（可空，默认 `application/octet-stream`）、`size`；响应 `id`、`name`、`content_type`、`size`、`upload_url`、`expires_at`。object key 为 `{owner_id}/{uuid}`，不进 JSON。预签名 15 分钟，绑定 `Content-Type` 和 `Content-Length`。申请时插入 `file.status = Pending`。
- `POST /file/{id}/complete`（JWT）确认上传：owner 才能操作。对 Garage 做 HeadObject，对象不存在 → `400 UploadIncomplete`；`Content-Length` 与申请时 `size` 不一致 → `400 SizeMismatch`；通过后把 `status` 写成 `Ready`。已是 `Ready` 则直接返回（含已有 `media`，不重探测、不出图）。响应 `id`、`owner_id`、`name`、`content_type`、`size`、`status`、`media`、`created_at`。缺失行 `404 FileNotFound`，非 owner `403 NotOwner`。
- `media` 表与 `file` 1:1（`file_id` PK，`ON DELETE CASCADE`），仍归 file 领域。有预览才插入一行。JSON 嵌在 `FileResponse.media`：`{ kind, width, height, duration_ms, thumbnail_id }` 或 `null`。`kind` 为 `Image` | `Video`。宽高是原文件显示尺寸（90°/270° 已对调）。`duration_ms` 仅 Video。`thumbnail_id` 是缩略图 `file.id`（JPEG，长边最多 320px），没有则为 `null`。探测或出图失败不阻断 Ready：没有视频流则不插行、`media: null`；探测成功但 ffmpeg/上传失败则仍插行、`thumbnail_id: null`。
- 探测 MIME：`image/jpeg` `image/png` `image/webp` `image/gif`（GIF 当 Image，忽略时长）和 `video/mp4` `video/quicktime` `video/webm`。complete 把对象流到临时文件，ffprobe 取流，ffmpeg 出第一帧。缩略图由服务端 `put_object` 写入 Garage 并插入 Ready 的 `file` 行，不走客户端 presign，不再探测。下载仍用 `GET /file/{id}/download`。
- `GET /file/{id}/download`（JWT）申请 GetObject 预签名 URL：任意登录用户，且 `status = Ready`。响应 `id`、`name`、`content_type`、`size`、`download_url`、`expires_at`。object key 不进 JSON。预签名 15 分钟，path-style，绑定 `response-content-type`（库里的 `content_type`）和 `response-content-disposition: attachment`（ASCII `filename` + RFC 5987 `filename*`，保存名为原 `name`）。不 HeadObject；对象已删则 Garage 对这条 URL 返回 404。仍是 Pending → `400 FileNotReady`；缺失行 `404 FileNotFound`。上传与 complete 仍仅 owner。
- `GET /file/{id}/content` 把 Ready 对象从 Garage 流式转给浏览器（预览）。任意登录用户。鉴权：`Authorization: Bearer`，或 cookie `kingei.access_token`（access JWT）。其它 `/file` 路由仍只认 Bearer。`Content-Type` 用库里的 `content_type`，`Content-Disposition: inline`（filename 编码与 download 相同），`Accept-Ranges: bytes`，`Cache-Control: private, max-age=86400`。请求带 `Range` 且对象支持则 206 + `Content-Range`，否则 200。不把对象读进内存。Pending → `400 FileNotReady`；缺失行 `404 FileNotFound`。保存文件仍走 `/download` 预签名，不走这条。
- `UserResponse` 的 `avatar` 是 file id 或 `null`，不嵌 File、不含 `download_url`。前端用 id 再调下载。
- `GET /friend` 的 `FriendResponse` 嵌 `user`（`UserResponse`）和好友关系 `created_at`。不摊平 `friend_id` / `username` / `email`。
- 把文件当图片用（用户/群头像）共用 `service::file::require_image`：属于当前用户、Ready、MIME 为 `image/jpeg`、`image/png`、`image/webp`、`image/gif`。失败 → `404 FileNotFound` / `403 NotOwner` / `400 FileNotReady` / `400 InvalidImage`。谁能把这张图挂到哪条记录，仍由各领域判断。
- 消息附件共用 `service::file::require_ready_file`：属于发送者且 `status = Ready`，不限 MIME。失败 → `404 FileNotFound` / `403 NotOwner` / `400 FileNotReady`，不创建消息。
- `POST /chat/{id}/message` 请求 `content`（可省略或为 `null`）和可选 `file_id`；`content` 为空且无 `file_id` → `400 EmptyContent`。`MessageResponse` 嵌 `file: FileResponse | null`，`GET /chat/{id}/message` 与 `message.created` SSE 同形。前端用 `file.media` 做气泡占位和封面（`thumbnail_id` 再调下载），`file.content_type` 区分类型。
- `PATCH /user/update` 可改 `username`、`email`、`avatar`。省略 `avatar` 不改头像；给 file id 则先 `require_image` 再写 `user.avatar`。只改 FK，不删 Garage 对象。
- `ChatResponse.avatar` 是 file id 或 `null`。私聊为 null。`PATCH /chat/{id}`（群主）可改 `name` 和/或 `avatar`；省略则不改。只改头像时走 `require_image`。Direct 仍 `CannotModifyDirect`。
- Access Token 15 分钟，Refresh Token 7 天。改时长只改 `src/utils/auth.rs` 里的常量。
- 受保护路由从 `Authorization: Bearer <token>` 取 access token，把 `claims.sub` 插入 `Extension<Uuid>`。`GET /file/{id}/content` 额外接受 cookie `kingei.access_token`；其它路由不认 cookie。
- 缺少/非法 header 视为 `UserError::InvalidCredentials`（401）。
- token 类型不符（用 refresh 当 access）视为 `UserError::InvalidToken`。
- 密码只存 bcrypt hash，`DEFAULT_COST`。响应里永远不要出现 `password` 或 `password_hash`。

---

## 8. 日志与注释

- 用户可见的启动/配置错误用中文。
- `tracing` 的事件名和字段用英文：`tracing::error!(?e, "database error")`。
- 代码注释用中文，只写非显而易见的约束，例如 token 有效期、为何跳过缺失用户。
- 不要为每个函数写“创建用户”“更新用户”这种复述式注释。
- 对外错误码用英文 PascalCase，不加中文句子。客户端自己映射文案。

---


