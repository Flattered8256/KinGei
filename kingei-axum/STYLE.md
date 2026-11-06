# Chat 项目风格规范

本文档描述本仓库的代码组织、命名、分层职责与提交约定。新增代码以本文为准；与旧代码不一致时，优先对齐 `friend` / `chat` 模块的写法，而不是早期 `user` 模块里的临时习惯。

原则：

- 结构先于技巧。业务按领域切开，每一层只做该层的事。
- 对外契约稳定。JSON 字段名、错误码、HTTP 状态码一旦对外暴露，不要随意改名。
- 内部错误不泄漏。数据库、JWT、bcrypt 的细节只写进日志，不返回给客户端。
- 少注释、写清楚。注释只解释“为什么”，不复述代码在做什么。

---

## 0. 依赖与实现取向

- **库优先**：能用成熟第三方库或框架自带能力完成的事，不手写。HTTP 消息解析、multipart、`Range`、`Content-Disposition`、MIME 嗅探、媒体探测、图片缩放、重试、限流、日期解析，一律先找现成的 crate，再考虑自己实现。
- **主流优先**：接口形状、配置方式、部署流程都对齐同类项目的常见做法，不自创协议或自创流程。新增对外接口前，先看成熟项目（Mattermost、Rocket.Chat、Immich、MinIO 生态）是怎么设计的。
- **选库标准**：优先官方 SDK（如 `aws-sdk-s3`）、维护活跃、支持 rustls、license 兼容、编译体积可接受。不要为了省一个依赖，把逻辑摊在业务代码里。
- **允许自己写**：库不存在、库明显更差（性能/正确性/安全）、或引入成本远大于收益时。这时在提交信息或注释里写清楚为什么不用现成库。
- **先提醒再动手**：如果需求偏离主流做法，或会导致为了特例把代码写复杂、用不上现成库，先说明主流做法和代价，得到确认后再实现。不要默默按需求写一套非主流实现。

---

## 1. 技术栈与目录

| 项 | 约定 |
| --- | --- |
| 语言 | Rust，`edition = "2024"` |
| Web | Axum 0.8 |
| 运行时 | Tokio（`macros` + `rt-multi-thread`） |
| 数据库 | SQLite 或 PostgreSQL + sqlx 0.9（cargo feature `sqlite` / `postgres`，默认 sqlite） |
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
  storage.rs           S3 兼容客户端（put / get / delete；本机 Garage 或云存储）
  web.rs               设置 STATIC_DIR 时托管静态页
migrations/sqlite/     SQLite 迁移脚本
migrations/postgres/   PostgreSQL 终态 schema
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

- 只接收 `DbPool`（或在方法内 `begin` 事务）。`DbPool` 按 feature 是 `SqlitePool` 或 `PgPool`。
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
| 已登录但无权 | 403 | `NotOwner`、`NotMember`、`NotRequestReceiver`、`AddFriendDisabled`、`CreateGroupDisabled` |
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

- 表名 **单数**：`user`、`friend`、`chat`、`chat_member`、`friend_request`、`file`、`instance`、`invite`。
- 列名 snake_case，与 model 字段相同。
- 用户主键：`TEXT` 存 UUID。
- 其余资源主键：`INTEGER PRIMARY KEY AUTOINCREMENT`，Rust 侧 `i64`。
- 时间：`TEXT NOT NULL`，Rust 侧 `DateTime<Utc>`。
- 枚举列：`TEXT`，sqlx `#[sqlx(rename_all = "PascalCase")]`，值为 `Pending` / `Direct` 这种 PascalCase。
- 索引：`idx_<表>_<列>`，例如 `idx_friend_user_id`。
- 迁移文件：`migrations/YYYYMMDDHHMMSS_动词_名词.sql`，例如 `20260902030000_create_chats.sql`。只追加，不改已经提交的迁移。

### 4.3 HTTP
- restful风格：成功直接返回资源，失败则放回code字段，错误代码用简短的英文。
- 顶层前缀：`/user`、`/friend`、`/chat`、`/event`、`/file`、`/instance`、`/admin`。
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
- 异步函数：仓储和服务用 `&self`，不要把 `DbPool` 传来传去。
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
- 占位符用 `$1` `$2` …（sqlx 的 SQLite 与 Postgres 都认），按 `.bind(...)` 顺序绑定。动态片段用 `QueryBuilder::<Db>`。
- 表名 `user` 写成 `"user"`（Postgres 保留字；SQLite 同样合法）。
- `INSERT` / `UPDATE` / `DELETE` 需要返回行时写 `RETURNING` 全列，列顺序与 `FromRow` 一致。
- 关键字大写，表名/列名小写：

```sql
SELECT id, user_id, friend_id, created_at
FROM friend
WHERE user_id = $1 AND friend_id = $2
```

- 需要原子性的多条语句放进同一个 repository 方法的事务，不要在 service 里“连续 await 两次写入”冒充事务。
- 不要 `SELECT *`。

---

## 7. 鉴权与配置

- 环境变量：`DATABASE_URL`、`PORT`、`JWT_SECRET`、`RUST_LOG`、`S3_ENDPOINT`、`S3_ACCESS_KEY`、`S3_SECRET_KEY`、`S3_BUCKET`、`S3_REGION`（缺省 `us-east-1`）、可选 `S3_PATH_STYLE`（缺省 `true`）、可选 `S3_MANAGE_BUCKET`（缺省 `true`）、可选 `MAX_UPLOAD_SIZE_MB`（缺省 `100`，单文件上限）、可选 `FFPROBE_PATH`（缺省 `ffprobe`）、可选 `FFMPEG_PATH`（缺省 `ffmpeg`）、可选 `JWT_SECRET_FILE`（缺省 `/kingei-data/jwt_secret`，仅 Docker 入口脚本读取，对应宿主机 `KinGei-data/jwt_secret`）。`.env` 不入库。Docker 中 `JWT_SECRET` 可空：入口脚本生成 32 字节 hex 写入该文件并 export。本机 `cargo run` 仍须设置非空 `JWT_SECRET`。启动时跑 `ffprobe -version` 和 `ffmpeg -version`，找不到则中文 `expect` 退出。
- 对象存储为 S3 兼容 API，只由服务端访问，浏览器不直连。本机 Garage：`S3_ENDPOINT` 如 `http://127.0.0.1:3900`，`S3_PATH_STYLE=true`。`S3_MANAGE_BUCKET=true` 时 `Storage::connect` 启动 `ListBuckets`，没有目标 bucket 则 `CreateBucket`。`S3_MANAGE_BUCKET=false`（云上）只 `HeadBucket`，不建桶、不改 bucket 配置。
- `GET /health` 不鉴权、不碰对象存储，固定 200。
- 单例 `instance` 表保存实例开关：`allow_add_friend`、`allow_create_group`（默认 true）、`initializing`（默认 true）和 `invite_only`（默认 false）。`GET /instance` 不鉴权，返回这四个 bool。`initializing` 表示还没有管理员：空库为 true，第一个用户在 `UserRepository::create` 同一事务里写成 Admin 并 `UPDATE instance SET initializing = false`。已有库迁移时若已有 user 则写成 false。不按次 COUNT user。`PATCH /admin/instance`（JWT + Admin）可改两个好友/建群开关和 `invite_only`，省略字段不改，空 `{}` 返回当前行，改不了 `initializing`。成员在 `allow_add_friend = false` 时 `POST /friend/request` → 403 `AddFriendDisabled`；`allow_create_group = false` 时 `POST /chat/group` → 403 `CreateGroupDisabled`。Admin 不受这两个开关影响。已有好友、私聊、群、邀请、同意申请仍可用。
- 单例 `invite` 表（id=1 CHECK）存全站一个邀请码 `code`，不预插入。`GET /admin/invite`（JWT + Admin）返回 `{ code }`，尚未生成 → 404 `InviteNotFound`。`POST /admin/invite` 无请求体，生成或替换 8 位码（字母表 `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`，Uuid 字节映射）。公开 `GET /instance` 不含码。`invite_only = true` 且非 initializing 时，`UserService::register` 把可选 `invite_code` trim + 大写后 `InviteRepository::find_by_code`（不改行）；缺码、空串、错码 → 404 `InviteNotFound`。`initializing` 时首个用户免码。`invite_only = false` 时忽略 `invite_code`。开启邀请码注册不自动生成码。
- SQLite 连接使用 `create_if_missing(true)`，库文件不存在则创建。Postgres 假定库已存在。`DATABASE_URL` 为 `sqlite:...` 或 `postgres://...`，须与编译 feature 一致。
- `POST /file`（JWT）直接上传：`multipart/form-data`，字段 `file`；文件名取该 part 的 `filename`，内容类型取该 part 的 `Content-Type`（缺省 `application/octet-stream`）。服务端边收边写临时文件，超过 `MAX_UPLOAD_SIZE_MB` → `413 FileTooLarge`。object key 为 `{owner_id}/{uuid}`，不进 JSON。响应 `FileResponse`：`id`、`owner_id`、`name`、`content_type`、`size`、`status`（直接 `Ready`）、`media`、`created_at`。空文件 `400 InvalidSize`，空名 `400 EmptyName`，超 255 字符 `400 FileNameTooLong`，缺 `file` 字段 `400 MissingFile`，multipart 解析失败 `400 InvalidMultipart`。
- `media` 表与 `file` 1:1（`file_id` PK，`ON DELETE CASCADE`），仍归 file 领域。有预览才插入一行。JSON 嵌在 `FileResponse.media`：`{ kind, width, height, duration_ms, thumbnail_id }` 或 `null`。`kind` 为 `Image` | `Video`。宽高是原文件显示尺寸（90°/270° 已对调）。`duration_ms` 仅 Video。`thumbnail_id` 是缩略图 `file.id`（JPEG，长边最多 320px），没有则为 `null`。探测或出图失败不阻断 Ready：没有视频流则不插行、`media: null`；探测成功但 ffmpeg/上传失败则仍插行、`thumbnail_id: null`。
- 探测 MIME：`image/jpeg` `image/png` `image/webp` `image/gif`（GIF 当 Image，忽略时长）和 `video/mp4` `video/quicktime` `video/webm`。上传先落临时文件，ffprobe 取流，ffmpeg 出第一帧。缩略图由服务端 `put_object` 写入对象存储并插入 Ready 的 `file` 行，不再探测。
- `GET /file/{id}/download` 把 Ready 对象流式转给浏览器（下载）。任意登录用户。鉴权：`Authorization: Bearer`，或 cookie `kingei.access_token`。`Content-Type` 用库里的 `content_type`，`Content-Disposition: attachment`（ASCII `filename` + RFC 5987 `filename*`，保存名为原 `name`），`Accept-Ranges: bytes`，`Cache-Control: private, max-age=86400`。请求带 `Range` 且对象支持则 206 + `Content-Range`，否则 200。不把对象读进内存。仍是 Pending → `400 FileNotReady`；缺失行 `404 FileNotFound`。
- `GET /file/{id}/content` 把 Ready 对象从对象存储流式转给浏览器（预览）。任意登录用户。鉴权：`Authorization: Bearer`，或 cookie `kingei.access_token`（access JWT）。`Content-Type` 用库里的 `content_type`，`Content-Disposition: inline`（filename 编码与 download 相同），`Accept-Ranges: bytes`，`Cache-Control: private, max-age=86400`。请求带 `Range` 且对象支持则 206 + `Content-Range`，否则 200。不把对象读进内存。Pending → `400 FileNotReady`；缺失行 `404 FileNotFound`。保存文件走 `/download`，不走这条。
- `UserResponse` 的 `avatar` 是 file id 或 `null`，不嵌 File。前端用 id 再调下载。
- `GET /friend` 的 `FriendResponse` 嵌 `user`（`UserResponse`）和好友关系 `created_at`。不摊平 `friend_id` / `username` / `email`。
- 把文件当图片用（用户/群头像）共用 `service::file::require_image`：属于当前用户、Ready、MIME 为 `image/jpeg`、`image/png`、`image/webp`、`image/gif`。失败 → `404 FileNotFound` / `403 NotOwner` / `400 FileNotReady` / `400 InvalidImage`。谁能把这张图挂到哪条记录，仍由各领域判断。
- 消息附件共用 `service::file::require_ready_file`：属于发送者且 `status = Ready`，不限 MIME。失败 → `404 FileNotFound` / `403 NotOwner` / `400 FileNotReady`，不创建消息。
- `POST /chat/{id}/message` 请求 `content`（可省略或为 `null`）和可选 `file_id`；`content` 为空且无 `file_id` → `400 EmptyContent`。`POST /chat/{id}/message/{message_id}` 请求体相同，对该条回复；目标不在本会话 → `404 MessageNotFound`，已软删 → `409 ReplyTargetDeleted`。`MessageResponse` 嵌 `file: FileResponse | null` 和 `reply_to: MessageResponse | null`（只一层，内层 `reply_to` 恒为 `null`；消息自身已删时 `reply_to` 也为 `null`）。`GET /chat/{id}/message`、发送/回复响应与 `message.created` SSE 同形。前端用 `file.media` 做气泡占位和封面（`thumbnail_id` 再调下载），`file.content_type` 区分类型。
- `PATCH /user/update` 可改 `username`、`email`、`avatar`。省略 `avatar` 不改头像；给 file id 则先 `require_image` 再写 `user.avatar`。只改 FK，不删对象存储里的文件。
- `ChatResponse.avatar` 是 file id 或 `null`。私聊为 null。`PATCH /chat/{id}`（群主）可改 `name` 和/或 `avatar`；省略则不改。只改头像时走 `require_image`。Direct 仍 `CannotModifyDirect`。
- Access Token 15 分钟，Refresh Token 7 天。改时长只改 `src/utils/auth.rs` 里的常量。
- 受保护路由从 `Authorization: Bearer <token>` 取 access token，把 `claims.sub` 插入 `Extension<Uuid>`。`GET /file/{id}/download` 和 `GET /file/{id}/content` 额外接受 cookie `kingei.access_token`；其它路由不认 cookie。
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


