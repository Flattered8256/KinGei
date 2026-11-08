# kingei-axum

后端规范。根目录 `AGENTS.md` 同样适用，本文件只写后端特有的部分。

## 技术栈

Rust edition 2024 · Axum 0.8 · Tokio · sqlx 0.9（feature `sqlite` 默认 / `postgres`）· serde · thiserror · JWT + bcrypt · tracing · aws-sdk-s3。

## 目录与依赖

每层一个门面文件 + 同名子目录。领域顺序：`user` `friend` `chat` `message` `file`。

```
src/
  main.rs          启动：环境变量、迁移、绑定端口
  state.rs         AppState，注入各 service
  router.rs        顶层路由组装
  router/<domain>.rs      路径 + 方法 + 是否鉴权
  handler/<domain>.rs     取参 → 调 service → 包装响应
  service/<domain>.rs     业务规则
  repository/<domain>.rs  SQL 与事务
  model/<domain>.rs       表实体 / 枚举
  serializer/<domain>.rs  请求体、响应体
  error/<domain>.rs       领域错误 + HTTP 映射
  middleware/      auth、admin
  utils/           JWT、媒体探测等工具函数
  storage.rs       S3 客户端
  web.rs           静态托管 + S3 反代
  event.rs         SSE 中心
migrations/sqlite|postgres
```

依赖只能向下，反向或跨层即错：

```
router → handler → service → repository
                       ↘ model
handler / service → serializer, error
repository → model
middleware → utils
```

- `repository` 不得出现 `use crate::{service, handler, serializer, error}`
- `handler` 不得直接调 repository、不得写 SQL
- `model` 不依赖 Axum / 请求类型
- `serializer` 不依赖 repository / service

## 各层写什么

**router** 只登记路径、方法、是否挂鉴权。公开用 `public_routes()`，登录用 `protected_routes()`（统一 `.layer(auth)`）；在 `router.rs` 里 `nest`，前缀用单数 `/user` `/friend` `/chat`。

**handler** 三步，不许有第四步：取参 → 调 service → `Ok(Json(..))` 或 `Ok(StatusCode::NO_CONTENT)`。变量名固定 `state`、`user_id`、`req`、`id`。校验、查库、拼装都不在这里。

**service** 持有 `Arc<XxxRepository>`，返回 `Result<T, AppError>`。权限、状态机、唯一性、空值校验都在这里；仓储返回 `None` 用 `ok_or` 转领域错误。私有辅助方法（`require_member`、`to_response`）放 `impl` 底部。

**repository** 只收 `DbPool`，返回 `Result<T, sqlx::Error>`；查不到给 `Option`，不决定 HTTP 语义。方法名用资源语义：`create` / `find` / `find_by_*` / `list*` / `update_*` / `delete`。

**model** 一表一结构，`#[derive(Debug, FromRow)]`，字段与列名一致。

**serializer** 请求 `*Request`、响应 `*Response`，用 `From<Model>` 映射；字段 snake_case，例外显式 `#[serde(rename)]`。

**error** 每领域一个枚举，包进 `AppError` 并用 `#[from]`。变体名 == `#[error("...")]` 文案 == 返回客户端的 `code`，body 只有 `{ "code": "..." }`。

| 场景 | 状态码 |
| --- | --- |
| 参数/规则不合法 | 400 |
| 未登录、凭据错、token 无效 | 401 |
| 已登录但无权 | 403 |
| 资源不存在 | 404 |
| 状态冲突（已存在、已在群里） | 409 |
| 基础设施失败 | 500，记 `tracing::error!`，正文 `InternalServerError` |

## 命名

| 种类 | 规则 | 例 |
| --- | --- | --- |
| 文件 / 模块 | snake_case 单数 | `friend.rs` |
| 类型 | PascalCase | `ChatService` |
| 错误枚举 | `<Domain>Error` | `ChatError` |
| 仓储 | `<Resource>Repository` | `FriendRequestRepository` |
| 表名 | 单数 | `chat_member` |
| 索引 | `idx_<表>_<列>` | `idx_friend_user_id` |
| 迁移 | `YYYYMMDDHHMMSS_动词_名词.sql` | `20260902030000_create_chats.sql` |

主键：用户 `TEXT`（UUID），其余 `INTEGER PRIMARY KEY AUTOINCREMENT`（`i64`）。时间 `TEXT` + `DateTime<Utc>`。枚举列 `TEXT` + `#[sqlx(rename_all = "PascalCase")]`。

## SQL

- `sqlx::query` / `query_as`，SQL 用 `r#"..."#`；占位符 `$1` 按顺序 `.bind()`；动态片段用 `QueryBuilder`。
- 不 `SELECT *`；表名 `"user"` 加引号（Postgres 保留字）。
- 要返回行时 `RETURNING` 全列，顺序与 `FromRow` 一致。
- 多步写入放进同一个 repository 方法的事务（`pool.begin()` + `&mut *tx`）；service 里连写两次 await 不算事务。
- 列表别在循环里 `find_by_id`，用 `find_by_ids` 或批量查询。

## 新增领域

按这套补齐，不要只加表或只加 handler：

1. `model/foo.rs` + `model.rs` 导出
2. `serializer/foo.rs` + 导出
3. `error/foo.rs` + 导出，并在 `AppError` 加 `#[from]` 变体
4. `repository/foo.rs` + 导出
5. `service/foo.rs` + 导出
6. `handler/foo.rs` + 导出
7. `router/foo.rs` + 在 `create_router` 里 `nest`
8. `state.rs` 加字段，并在 `AppState::new` 注入

## 其他

- 注释只写"为什么"，不复述代码；对外错误码用英文 PascalCase，不加中文句子。
- 启动期缺配置用 `expect` 且中文提示；请求路径上的错误用 `Result + ?`，不 `unwrap`。
- 响应里永远不出现 `password` / `password_hash`。
- 对外契约（字段名、错误码、状态码）改动按破坏性变更处理。

## 完成前

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo build
cargo build --no-default-features --features postgres   # 动了 sqlx 或迁移
```
