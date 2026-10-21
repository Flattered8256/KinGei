<script setup lang="ts">
import type { Chat, Message } from '~/types/api'
import { formatChatTime } from '~/utils/display'

const props = defineProps<{
  chat: Chat
}>()

const { t, locale } = useI18n()
const { user } = useAuth()
const users = useUserDirectory()
const messages = useMessages()
const scroller = ref<HTMLElement | null>(null)
const stickToBottom = ref(true)

const myId = computed(() => user.value?.id ?? '')

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

function onScroll() {
  const el = scroller.value
  if (!el) {
    return
  }
  stickToBottom.value = el.scrollHeight - el.scrollTop - el.clientHeight < 80
  if (el.scrollTop < 40) {
    const previousHeight = el.scrollHeight
    void messages.loadOlder().then(() => {
      nextTick(() => {
        if (scroller.value) {
          scroller.value.scrollTop = scroller.value.scrollHeight - previousHeight
        }
      })
    })
  }
}

function scrollToBottom() {
  nextTick(() => {
    const el = scroller.value
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  })
}

watch(
  () => messages.items.value.length,
  (len, prev) => {
    if (stickToBottom.value || (prev ?? 0) === 0) {
      scrollToBottom()
    }
  },
)

watch(
  () => props.chat.id,
  () => {
    stickToBottom.value = true
    scrollToBottom()
  },
)

onMounted(scrollToBottom)
</script>

<template>
  <div
    ref="scroller"
    class="thread"
    @scroll="onScroll"
  >
    <button
      v-if="messages.hasMore.value && messages.items.value.length"
      type="button"
      class="more"
      :disabled="messages.loadingMore.value"
      @click="messages.loadOlder()"
    >
      {{ $t('chat.loadMore') }}
    </button>
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
    <div
      v-for="message in messages.items.value"
      :key="message.id"
      class="row"
      :class="{ own: isOwn(message) }"
    >
      <Avatar
        v-if="!isOwn(message)"
        :name="sender(message)?.username"
        :seed="message.sender_id"
        :size="36"
      />
      <div class="bubble-wrap">
        <div class="meta">
          <span
            v-if="!isOwn(message)"
            class="name"
          >{{ sender(message)?.username ?? $t('app.unknownUser') }}</span>
          <span class="time">{{ timeOf(message) }}</span>
        </div>
        <div class="bubble">
          {{ message.content }}
        </div>
        <div
          v-if="readers(message).length"
          class="reads"
        >
          <Avatar
            v-for="member in readers(message)"
            :key="member.user.id"
            :name="member.user.username"
            :seed="member.user.id"
            :size="16"
          />
        </div>
      </div>
      <Avatar
        v-if="isOwn(message)"
        :name="user?.username"
        :seed="user?.id"
        :size="36"
      />
    </div>
  </div>
</template>

<style scoped>
.thread {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 16px 20px 8px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.center {
  text-align: center;
  padding: 24px;
}

.more {
  align-self: center;
  color: var(--color-primary);
  font-size: 13px;
  padding: 4px 8px;
}

.row {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  max-width: 72%;
}

.row.own {
  align-self: flex-end;
  flex-direction: row;
}

.bubble-wrap {
  min-width: 0;
}

.row.own .bubble-wrap {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.meta {
  display: flex;
  gap: 8px;
  align-items: baseline;
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
  color: var(--color-muted);
  font-size: 12px;
}

.bubble {
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
