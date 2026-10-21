<script setup lang="ts">
import { VList } from 'virtua/vue'
import type { VListHandle } from 'virtua/vue'
import type { Chat, Message } from '~/types/api'
import { formatChatTime } from '~/utils/display'

const props = defineProps<{
  chat: Chat
}>()

const { t, locale } = useI18n()
const { user } = useAuth()
const users = useUserDirectory()
const messages = useMessages()
const NEAR_BOTTOM = 80
const FLUSH_BOTTOM = 1

const listRef = ref<VListHandle | null>(null)
const stickToBottom = ref(true)
const pendingBottom = ref(false)
const flushBottom = ref(true)
const shift = ref(false)
const ready = ref(false)

const myId = computed(() => user.value?.id ?? '')
const showJump = computed(() => !stickToBottom.value && messages.items.value.length > 0)

const prevIds = {
  first: null as number | null,
  last: null as number | null,
}

function sender(message: Message) {
  const member = props.chat.member.find(item => item.user.id === message.sender_id)?.user
  return member ?? users.get(message.sender_id)
}

function isOwn(message: Message) {
  return message.sender_id === myId.value
}

function readers(message: Message) {
  if (!isOwn(message)) {
    return []
  }
  return props.chat.member.filter((member) => {
    if (member.user.id === myId.value) {
      return false
    }
    return (member.last_read_message_id ?? 0) >= message.id
  })
}

function timeOf(message: Message) {
  return formatChatTime(message.created_at, locale.value, t('app.yesterday'))
}

function scrollToLatest() {
  nextTick(() => {
    const last = messages.items.value.length - 1
    if (last < 0) {
      return
    }
    listRef.value?.scrollToIndex(last, { align: 'end' })
    ready.value = true
  })
}

function jumpToBottom() {
  stickToBottom.value = true
  pendingBottom.value = true
  flushBottom.value = true
  scrollToLatest()
}

function distanceFromBottom(offset: number, handle: VListHandle) {
  return handle.scrollSize - offset - handle.viewportSize
}

function onScroll(offset: number) {
  const handle = listRef.value
  if (!handle) {
    return
  }
  const distance = distanceFromBottom(offset, handle)
  flushBottom.value = distance <= FLUSH_BOTTOM
  const nearBottom = distance < NEAR_BOTTOM
  if (pendingBottom.value) {
    if (nearBottom) {
      pendingBottom.value = false
      stickToBottom.value = true
    }
  }
  else {
    stickToBottom.value = nearBottom
  }
  if (!ready.value || messages.loadingMore.value || !messages.hasMore.value) {
    return
  }
  if (offset < NEAR_BOTTOM) {
    void messages.loadOlder()
  }
}

function onResize() {
  // First upward scroll measures new rows; pin only if still flush, not merely near.
  if (pendingBottom.value || flushBottom.value) {
    scrollToLatest()
  }
}

watch(
  () => messages.items.value,
  (items) => {
    const firstId = items[0]?.id ?? null
    const lastId = items.at(-1)?.id ?? null
    const prepended = prevIds.first != null && firstId !== prevIds.first && lastId === prevIds.last
    const appended = prevIds.last != null && lastId !== prevIds.last && firstId === prevIds.first
    shift.value = prepended
    if (appended) {
      const last = items.at(-1)
      if (stickToBottom.value || last?.sender_id === myId.value) {
        stickToBottom.value = true
        pendingBottom.value = true
      }
    }
    prevIds.first = firstId
    prevIds.last = lastId
  },
  { flush: 'sync', immediate: true },
)

watch(
  () => {
    const items = messages.items.value
    return {
      length: items.length,
      firstId: items[0]?.id ?? null,
      lastId: items.at(-1)?.id ?? null,
    }
  },
  (curr, prev) => {
    const prepended = !!prev
      && curr.firstId !== prev.firstId
      && curr.lastId === prev.lastId
      && curr.length > prev.length
    if (prepended) {
      return
    }
    if (pendingBottom.value || stickToBottom.value || !prev || prev.length === 0) {
      scrollToLatest()
    }
  },
  { flush: 'post' },
)

watch(
  () => props.chat.id,
  () => {
    stickToBottom.value = true
    pendingBottom.value = true
    flushBottom.value = true
    shift.value = false
    ready.value = false
    prevIds.first = null
    prevIds.last = null
    scrollToLatest()
  },
)

watch(
  () => messages.sending.value,
  (sending) => {
    if (sending) {
      jumpToBottom()
    }
  },
)

onMounted(scrollToLatest)
</script>

<template>
  <div class="thread">
    <p
      v-if="messages.loading.value"
      class="muted center"
    >
      {{ $t('app.loading') }}
    </p>
    <p
      v-else-if="!messages.items.value.length"
      class="muted center"
    >
      {{ $t('chat.noMessages') }}
    </p>
    <VList
      v-else
      ref="listRef"
      class="list"
      :data="messages.items.value"
      :shift="shift"
      @scroll="onScroll"
      @resize="onResize"
    >
      <template #default="{ item, index }">
        <div
          :key="item.id"
          class="row"
          :class="{
            own: isOwn(item),
            lead: index === 0,
            tail: index === messages.items.value.length - 1,
          }"
        >
          <Avatar
            v-if="!isOwn(item)"
            :name="sender(item)?.username"
            :seed="item.sender_id"
            :size="36"
          />
          <div class="bubble-wrap">
            <div class="meta">
              <span
                class="name truncate"
                :title="sender(item)?.username ?? $t('app.unknownUser')"
              >{{ sender(item)?.username ?? $t('app.unknownUser') }}</span>
              <span class="time">{{ timeOf(item) }}</span>
            </div>
            <div class="bubble">
              {{ item.content }}
            </div>
            <div
              v-if="readers(item).length"
              class="reads"
            >
              <Avatar
                v-for="member in readers(item)"
                :key="member.user.id"
                :name="member.user.username"
                :seed="member.user.id"
                :size="16"
              />
            </div>
          </div>
          <Avatar
            v-if="isOwn(item)"
            :name="user?.username"
            :seed="user?.id"
            :size="36"
          />
        </div>
      </template>
    </VList>
    <button
      v-if="showJump"
      type="button"
      class="to-bottom"
      :aria-label="$t('chat.scrollToBottom')"
      :title="$t('chat.scrollToBottom')"
      @click="jumpToBottom"
    >
      <AppIcon name="down" :size="20" />
    </button>
  </div>
</template>

<style scoped>
.thread {
  position: relative;
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.center {
  margin: auto;
  text-align: center;
  padding: 24px;
}

.list {
  flex: 1;
  min-height: 0;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.list::-webkit-scrollbar {
  display: none;
  width: 0;
  height: 0;
}

.to-bottom {
  position: absolute;
  right: 16px;
  bottom: 16px;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--bg-panel);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-card);
}

.to-bottom:hover {
  background: var(--bg-hover);
}

.row {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  justify-content: flex-start;
  padding: 7px 20px;
}

.row.lead {
  padding-top: 16px;
}

.row.tail {
  padding-bottom: 16px;
}

.row.own {
  justify-content: flex-end;
}

.bubble-wrap {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  min-width: 0;
  max-width: 72%;
}

.row.own .bubble-wrap {
  align-items: flex-end;
}

.meta {
  display: flex;
  gap: 8px;
  align-items: baseline;
  width: fit-content;
  max-width: 100%;
  min-width: 0;
  margin-bottom: 4px;
}

.row.own .meta {
  flex-direction: row-reverse;
}

.name {
  color: var(--color-primary);
  font-size: 13px;
  font-weight: 600;
}

.time {
  flex-shrink: 0;
  color: var(--color-muted);
  font-size: 12px;
}

.bubble {
  width: fit-content;
  max-width: 100%;
  padding: 8px 12px;
  border-radius: 12px;
  background: var(--color-bubble-other);
  white-space: pre-wrap;
  word-break: break-word;
}

.row.own .bubble {
  background: var(--color-bubble-own);
}

.reads {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
  margin-top: 4px;
}
</style>
