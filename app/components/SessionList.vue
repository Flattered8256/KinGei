<script setup lang="ts">
import { formatChatTime, initials, messagePreviewText } from '~/utils/display'

const { t, locale } = useI18n()
const chats = useChats()
const { open } = useAppActions()
const { canCreateGroup } = useInstance()
const route = useRoute()
const query = ref('')
const showSearch = ref(false)
const showMenu = ref(false)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) {
    return chats.chats.value
  }
  return chats.chats.value.filter((chat) => {
    const title = chats.titleOf(chat).toLowerCase()
    const preview = chats.previews.value[chat.id]
    const content = preview?.content.toLowerCase() ?? ''
    const fileName = preview?.file?.name.toLowerCase() ?? ''
    const label = preview ? previewText(chat.id).toLowerCase() : ''
    return title.includes(q) || content.includes(q) || fileName.includes(q) || label.includes(q)
  })
})

function avatarName(chatId: number, title: string) {
  const chat = chats.chats.value.find(item => item.id === chatId)
  if (!chat || chat.type === 'Group') {
    return title
  }
  return chats.peerOf(chat)?.username ?? title
}

function avatarSeed(chatId: number) {
  const chat = chats.chats.value.find(item => item.id === chatId)
  if (!chat) {
    return ''
  }
  if (chat.type === 'Group') {
    return `group-${chat.id}`
  }
  return chats.peerOf(chat)?.id ?? String(chat.id)
}

function avatarFileId(chatId: number) {
  const chat = chats.chats.value.find(item => item.id === chatId)
  if (!chat) {
    return null
  }
  if (chat.type === 'Group') {
    return chat.avatar
  }
  return chats.peerOf(chat)?.avatar ?? null
}

function groupLetter(title: string) {
  return initials(title) === '?' ? '#' : initials(title)
}

function isActive(id: number) {
  return String(route.params.id) === String(id)
}

function previewText(id: number) {
  const message = chats.previews.value[id]
  if (!message) {
    return ''
  }
  return messagePreviewText(message, {
    image: t('chat.previewImage'),
    video: t('chat.previewVideo'),
    file: t('chat.previewFile'),
  })
}

function previewTime(id: number, createdAt: string) {
  const iso = chats.previews.value[id]?.created_at ?? createdAt
  return formatChatTime(iso, locale.value, t('app.yesterday'))
}

function toggleSearch() {
  showSearch.value = !showSearch.value
  if (!showSearch.value) {
    query.value = ''
  }
}

function pickGroup() {
  showMenu.value = false
  open('group')
}

onClickOutsideMenu()

function onClickOutsideMenu() {
  if (!import.meta.client) {
    return
  }
  function handle(event: MouseEvent) {
    const target = event.target as HTMLElement | null
    if (!target?.closest('.plus-wrap')) {
      showMenu.value = false
    }
  }
  onMounted(() => document.addEventListener('click', handle))
  onBeforeUnmount(() => document.removeEventListener('click', handle))
}
</script>

<template>
  <section class="sessions">
    <header class="head">
      <div class="brand">
        <Avatar
          name="K"
          :size="32"
          :hash="false"
        />
        <strong>{{ $t('app.name') }}</strong>
      </div>
      <div class="actions">
        <button
          type="button"
          class="icon-btn"
          :title="$t('app.search')"
          @click="toggleSearch"
        >
          <AppIcon name="search" :size="20" />
        </button>
        <div
          v-if="canCreateGroup"
          class="plus-wrap"
        >
          <button
            type="button"
            class="icon-btn"
            :title="$t('chat.new')"
            @click="showMenu = !showMenu"
          >
            <AppIcon name="plus" :size="20" />
          </button>
          <div
            v-if="showMenu"
            class="menu"
          >
            <button
              type="button"
              @click="pickGroup"
            >
              {{ $t('chat.createGroup') }}
            </button>
          </div>
        </div>
      </div>
    </header>
    <div
      v-if="showSearch"
      class="search"
    >
      <input
        v-model="query"
        class="input"
        :placeholder="$t('chat.searchPlaceholder')"
      >
    </div>
    <div class="list">
      <p
        v-if="!filtered.length && !chats.loading.value"
        class="muted pad"
      >
        {{ $t('chat.noChats') }}
      </p>
      <NuxtLink
        v-for="chat in filtered"
        :key="chat.id"
        :to="`/chat/${chat.id}`"
        class="item"
        :class="{ active: isActive(chat.id) }"
      >
        <Avatar
          :name="chat.type === 'Group' ? groupLetter(chats.titleOf(chat)) : avatarName(chat.id, chats.titleOf(chat))"
          :seed="avatarSeed(chat.id)"
          :file-id="avatarFileId(chat.id)"
          :size="40"
        />
        <div class="meta">
          <div class="body">
            <strong class="truncate">{{ chats.titleOf(chat) }}</strong>
            <span class="preview truncate">{{ previewText(chat.id) }}</span>
          </div>
          <div class="trail">
            <span class="time">{{ previewTime(chat.id, chat.created_at) }}</span>
            <UnreadBadge
              v-if="chat.unread_count > 0"
              :value="chat.unread_count"
            />
          </div>
        </div>
      </NuxtLink>
    </div>
    <KgOverlay v-if="chats.loading.value && !chats.chats.value.length" />
  </section>
</template>

<style scoped>
.sessions {
  position: relative;
  display: flex;
  flex-direction: column;
  width: var(--session-width);
  min-width: var(--session-width);
  border-right: 1px solid var(--color-border);
  min-height: 0;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 8px 16px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
}

.actions {
  display: flex;
  align-items: center;
}

.plus-wrap {
  position: relative;
}

.menu {
  position: absolute;
  right: 0;
  top: 42px;
  z-index: 5;
  display: flex;
  flex-direction: column;
  min-width: 160px;
  padding: 6px;
  background: var(--bg-panel);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.menu button {
  padding: 8px 10px;
  border-radius: 6px;
  text-align: left;
}

.menu button:hover {
  background: var(--bg-hover);
}

.search {
  padding: 0 12px 8px;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  overflow: auto;
  padding: 4px 8px 12px;
}

.item {
  display: flex;
  gap: 10px;
  padding: 10px;
  border-radius: var(--radius-lg);
  color: inherit;
}

.item.active,
.item:hover {
  background: var(--bg-active);
}

.meta {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.trail {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  gap: 4px;
}

.preview,
.time {
  color: var(--color-muted);
  font-size: 12px;
}

.pad {
  padding: 12px 16px;
}

@media (max-width: 899px) {
  .sessions {
    flex: 1;
    width: 100%;
    min-width: 0;
    border-right: 0;
  }
}
</style>
