# kingei-nuxt

前端规范。根目录 `AGENTS.md` 同样适用，本文件只写前端特有的部分。

## 技术栈与约束

Nuxt 4 · Vue 3 `<script setup>` · TypeScript · element-plus · @nuxtjs/i18n · virtua · `ssr: false`（纯 CSR）。

- 没有 SSR：不要用只在服务端生效的写法（`useAsyncData` 的服务端缓存、`useRequestHeaders` 等）；浏览器侧副作用放 `onMounted` 或判 `import.meta.client`。
- 状态管理见「状态管理」一节：目标形态是 Pinia，当前是 `useState`（属已知偏差）。

## 目录与分层

```
app/
  pages/            路由页面：编排，不写请求细节
  layouts/          app（主框架）、auth（登录注册）
  middleware/       auth、guest、admin
  components/       Kg* 基础件 + 业务组件
  composables/      状态与流程（use*）
  utils/api/        后端接口封装（唯一出口）
  utils/display.ts  纯函数：格式化、标题、附件类型
  types/api.ts      后端契约类型 + 错误码 + 事件名
  assets/css/main.css  设计 token
i18n/locales/       zh-CN.json、en.json
```

依赖只能单向，不能反向：

```
pages / components → composables → utils/api → types
                          ↘ utils/display（纯函数）
```

- 页面和组件**不得直接** `$fetch` / `ofetch`，一律经 `utils/api/*`。
- `utils/api/*` 只依赖 `http.ts` 和 `types/api.ts`，不持有 Vue 状态、不弹 toast。
- `composables/*` 管状态、副作用与错误提示，可以互相调用。
- `utils/display.ts` 保持纯函数，不 import Vue、不 import composable。

## 状态管理

- 【规定】领域状态用 Pinia（`stores/<domain>.ts`，setup store 语法）。理由：官方推荐的 Vue 状态方案，有 devtools 时间线、编译期 store id、插件生态；本项目已有 440 行的消息状态和 SSE 驱动的归约逻辑，超出 `useState` 的适用范围。
- 【规定】composable 保留为薄壳：`useChats()` 等签名不变，内部转调 store；页面和组件不直接 import store。
- 【现状】尚未迁移：`chats` / `messages` / `friends` / `auth` / `instance` / `users` 仍是 `useState` 实现。迁移顺序 `chats` → `messages` → `friends` → 其余。
- 【例外】纯 UI 标志继续用 `useState`：`theme.*`、`ui.isMobile`、`ui.modal`、`sse.status`、`app.ready`。
- 迁移完成前**不要**再往 `useState` 加新的领域状态；需要新状态时先提议接入 Pinia（依赖变更，需用户确认）。

迁移期间，现有 `useState` 代码按以下约定维护：

- key 用 `域.字段`：`chats.list`、`auth.user`。
- composable 返回 `ref` / `computed` + 动作函数，不返回裸对象。
- 只在跨页面或跨组件共享时用 `useState`；页面局部状态用 `ref`。
- 每个领域 composable 提供 `reset()`，登出时由 `useAppSync.resetAll()` 统一清。

## 请求与错误

- 只调 `apiFetch<T>` / `apiFetchVoid`（`utils/api/http.ts`）。
- 401 刷新与并发去重已在 `http.ts` 内部处理，业务层不要重复写刷新逻辑。
- 错误统一是 `ApiError`（带 `status` + `code`）；给用户看的文案用 `useApiI18n().fromError(err)`，不要直接展示 `err.message`。
- 需要静默失败的分支必须写一行注释说明原因（例：会话列表的预览失败不影响列表本身）。
- 新增接口的顺序：后端落地 → `utils/api/` 加一个薄函数 → `types/api.ts` 补类型。

## 类型

- `types/api.ts` 是后端 serializer 的镜像，字段名保持 snake_case 一致。
- 新错误码要同时加进 `API_ERROR_CODES` 和 `i18n/locales/*.json` 的 `error.<Code>`。
- 新 SSE 事件要同时加进 `CLIENT_EVENT_NAMES`、`ClientEvent` 联合类型，并在对应 composable 的 `ingest` 里处理。

## 组件

- 一律 `<script setup lang="ts">`；props / emits 用类型声明（`defineProps<{ ... }>()`）。
- `Kg*` 是设计系统基础件，只做薄封装；业务组件按领域命名（`Message*`、`Friend*`、`Chat*`）。
- 需要基础件时先看 element-plus 有没有现成的；`Kg*` 只承担"设计稿必须统一"的那一层，不重复实现已有组件。
- 单文件超 300 行：先抽 composable，再考虑拆子组件。

## 样式

- 只用 `main.css` 的 CSS 变量（`--bg-*`、`--color-*`、`--radius-*`、`--shadow-*`），不写死颜色。
- 深色主题靠 `[data-theme='dark']` 覆盖同一批变量，所以组件里不允许出现 `#fff` 这类魔法值（覆盖第三方库样式除外）。
- 组件内用 `<style scoped>`；确实要跨组件复用的才进 `main.css`。

## 路由与页面

- `definePageMeta({ layout, middleware })` 声明布局与鉴权：登录后用 `layout: 'app'` + `middleware: ['auth']`，管理页再加 `'admin'`。
- 页面只做编排：取数据 → 传 props → 处理子组件事件。
- 路由文件按路径命名，嵌套路由放同名目录（`pages/chat/[id].vue`）。

## i18n

- 所有用户可见文案走 `t()`，key 同时写进 `i18n/locales/zh-CN.json` 和 `en.json`。
- 错误码映射 `error.<Code>`，枚举映射 `enum.*`；不要拼句子，用带占位符的 key。
- 日期与时间用 `utils/display.ts` 的格式化函数，locale 取 `useI18n().locale`。

## 已知偏差（现状，不要照抄）

以下是当前代码里存在、但**不构成参考**的写法。发现新的偏差：登记到这里并说明是待修还是有意保留，不要默认沿用。

- 领域状态用 `useState` 而非 Pinia（见「状态管理」，待迁移）。
- `KgSwitch` / `KgModal` 是手写基础件，element-plus 有 `ElSwitch` / `ElDialog`；新基础件先查 element-plus，不要重复手写。
- 组件里有 12 处硬编码颜色（`#fff` / `#000` / `#22ccee`，分布在 `InviteFriendModal`、`Avatar`、`MessageComposer`、`MessageAttachment`、`MediaLightbox`、`KgSwitch`），属遗留；新样式必须走 CSS 变量。
- `useInstance` 的 state key 是无域前缀的 `instance`（其余均为 `域.字段`），待统一。

## 完成前

```bash
pnpm typecheck
```

lint 尚未配置（没有 eslint）。新增代码对齐现有风格：2 空格缩进、单引号、不写分号。
