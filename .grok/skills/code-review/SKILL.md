---
name: code-review
description: 按本仓库规范审查一次改动，输出"文件:行 + 违反的规则 + 具体改法"，并按严重度分级。适用于功能完成后的自检、提交前评审，或用户说"看看这段代码合不合规范"。
whenToUse: 一次实现结束后、准备提交前，或用户要求 review 代码。
---

# 代码评审

只评审**本次改动**，不评审既有代码，不评价个人偏好。规则来源：根 `AGENTS.md`、`kingei-axum/AGENTS.md`（分层、错误映射、命名、SQL）。

评审前先确认改动范围：`git status --short` + `git diff`。范围之外的问题只记一句，不改。

## 检查项（按严重度）

### P0 — 必须改

- **分层违规**：`src/repository/*` 出现 `use crate::service|handler|serializer|error`；`src/handler/*` 里查库（`state.*_repository`）、做校验或业务判断；`src/serializer/*` 依赖 `repository` 或 `service`。
- **契约破坏**：JSON 字段名、错误码、HTTP 状态码发生变化而未在回复中说明。
- **内部错误泄漏**：把 `sqlx::Error`、bcrypt、JWT 的细节返回给客户端；`password` / `password_hash` 进入任何 Response 类型。
- **请求路径上的 `unwrap` / `expect` / 吞错**（`let _ = ...`、空 `catch`）。

### P1 — 应该改

- **规则重复**：同一条业务规则出现在 handler、service、repository 两处以上。
- **库优先失败**：手写了库/框架已有能力（开关、弹窗、加载态、日期格式化、请求封装、`Range` 解析、multipart 拼装）。
- **事务缺失**：service 里连续两次写入冒充原子操作，或跨表写入没放在同一个 repository 方法的事务内。
- **N+1**：循环里 `find_by_id`，应改 `find_by_ids`。
- **复制既有反模式**：新代码沿用了规范里登记为「已知偏差」的写法（手写基础件、硬编码颜色、把领域状态塞进 `useState` 等）。对照 `kingei-nuxt/AGENTS.md` 的「已知偏差」清单检查。
- **前端越层**：组件/页面里直接 `$fetch`/`fetch`，绕过 `app/utils/api/*`；页面里存跨页业务状态而非放进 `composables`。
- **硬编码**：样式里写死颜色/间距（应使用 `app/assets/css/main.css` 的 token）；用户可见文案不走进 i18n。

### P2 — 可改

- 文件超过规模阈值（后端领域 > 200 行未拆、前端单文件 > 300 行未抽 composable）。
- 命名不符合规范（文件/类型/函数/错误枚举/仓储）。
- 复述式注释、提交信息格式错误。

## 输出格式

```
| 级别 | 文件:行 | 违反 | 依据 | 改法 |
|---|---|---|---|---|
| P0 | src/handler/chat.rs:42 | handler 里判断成员关系 | axum AGENTS.md · handler 层 | 移到 ChatService::require_member |

结论：可以提交 / 需修改 N 处（P0 x 个、P1 y 个）
```

同一类问题合并成一行，标出所有位置，不要一个问题刷十行。

## 不做

- 不提"可以更优雅""建议重构整体架构"这类无法执行的建议。
- 不顺手改代码；除非用户明确要求，评审只出结论。
- 不把格式争议当问题（rustfmt 管的事不归评审管）。
- 不为凑数提规范里没写的偏好。
