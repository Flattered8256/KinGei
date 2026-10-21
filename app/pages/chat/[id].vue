<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

import { getChat } from '~/utils/api/chat'

const route = useRoute()
const { isMobile } = useIsMobile()
const chats = useChats()
const messages = useMessages()
const showMembers = ref(false)
const missing = ref(false)

const chatId = computed(() => Number(route.params.id))
const chat = computed(() => chats.chats.value.find(item => item.id === chatId.value) ?? null)

watch(
  chatId,
  async (id) => {
    showMembers.value = false
    missing.value = false
    if (!Number.isFinite(id)) {
      return
    }
    void messages.open(id)
    if (chats.chats.value.some(item => item.id === id)) {
      return
    }
    try {
      chats.replaceChat(await getChat(id))
    }
    catch {
      missing.value = true
    }
  },
  { immediate: true },
)

const title = computed(() => (chat.value ? chats.titleOf(chat.value) : ''))
const isGroup = computed(() => chat.value?.type === 'Group')
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
    class="empty-state"
  >
    <p>{{ chats.loading.value && !missing ? $t('app.loading') : $t('error.ChatNotFound') }}</p>
  </div>
</template>

<style scoped>
.room {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
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
