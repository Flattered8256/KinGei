<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
  key: 'chat-room',
})

import { getChat } from '~/utils/api/chat'
import { initials } from '~/utils/display'

const route = useRoute()
const { isMobile } = useIsMobile()
const chats = useChats()
const messages = useMessages()
const showMembers = ref(false)
const missing = ref(false)
const fetching = ref(false)

const chatId = computed(() => Number(route.params.id))
const chat = computed(() => chats.chats.value.find(item => item.id === chatId.value) ?? null)

watch(
  chatId,
  async (id) => {
    showMembers.value = false
    missing.value = false
    if (!Number.isFinite(id)) {
      fetching.value = false
      return
    }
    void messages.open(id)
    if (chats.chats.value.some(item => item.id === id)) {
      fetching.value = false
      return
    }
    fetching.value = true
    try {
      chats.replaceChat(await getChat(id))
    }
    catch {
      if (chatId.value === id) {
        missing.value = true
      }
    }
    finally {
      if (chatId.value === id) {
        fetching.value = false
      }
    }
  },
  { immediate: true },
)

const showRoomOverlay = computed(() =>
  !chat.value && !missing.value && (fetching.value || chats.loading.value),
)

const title = computed(() => (chat.value ? chats.titleOf(chat.value) : ''))
const isGroup = computed(() => chat.value?.type === 'Group')

const headerAvatar = computed(() => {
  const current = chat.value
  if (!current) {
    return { name: '', seed: '', fileId: null as number | null }
  }
  if (current.type === 'Group') {
    const letter = initials(title.value)
    return {
      name: letter === '?' ? '#' : letter,
      seed: `group-${current.id}`,
      fileId: current.avatar,
    }
  }
  const peer = chats.peerOf(current)
  return {
    name: peer?.username ?? title.value,
    seed: peer?.id ?? String(current.id),
    fileId: peer?.avatar ?? null,
  }
})
</script>

<template>
  <div
    v-if="chat"
    class="room"
  >
    <header class="top">
      <NuxtLink
        v-if="isMobile"
        to="/chat"
        class="icon-btn"
        :aria-label="$t('app.back')"
      >
        <AppIcon name="back" />
      </NuxtLink>
      <Avatar
        :name="headerAvatar.name"
        :seed="headerAvatar.seed"
        :file-id="headerAvatar.fileId"
        :size="32"
      />
      <div class="who">
        <strong>{{ title }}</strong>
        <span
          v-if="isGroup"
          class="muted"
        >{{ $t('chat.members') }} {{ chat.member.length }}</span>
      </div>
      <button
        v-if="isGroup"
        type="button"
        class="icon-btn"
        :class="{ active: showMembers }"
        :title="$t('chat.members')"
        :aria-expanded="showMembers"
        @click="showMembers = !showMembers"
      >
        <AppIcon name="friends" />
      </button>
    </header>
    <div class="main">
      <div class="conversation">
        <MessageThread :chat="chat" />
        <MessageComposer />
      </div>
      <ChatMemberPanel
        v-if="isGroup && showMembers && !isMobile"
        :chat="chat"
      />
      <div
        v-if="isGroup && showMembers && isMobile"
        class="drawer-backdrop"
        @click="showMembers = false"
      >
        <div
          class="drawer"
          @click.stop
        >
          <ChatMemberPanel :chat="chat" />
        </div>
      </div>
    </div>
  </div>
  <div
    v-else
    class="empty-state room-missing"
  >
    <KgOverlay v-if="showRoomOverlay" />
    <p v-else>{{ $t('error.ChatNotFound') }}</p>
  </div>
</template>

<style scoped>
.room {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.room-missing {
  position: relative;
  flex: 1;
  width: 100%;
}

.top {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 56px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
}

.who {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.main {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
}

.conversation {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.drawer-backdrop {
  position: absolute;
  inset: 0;
  z-index: 10;
  display: flex;
  justify-content: flex-end;
  background: rgba(15, 23, 42, 0.35);
}

.drawer {
  width: min(280px, 85%);
  height: 100%;
  min-height: 0;
  display: flex;
  background: var(--bg-panel);
  box-shadow: var(--shadow-modal);
}

.drawer :deep(.panel) {
  width: 100%;
  min-width: 0;
}
</style>
