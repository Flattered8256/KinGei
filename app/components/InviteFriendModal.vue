<script setup lang="ts">
import type { Friend } from '~/types/api'

const props = defineProps<{
  open: boolean
  friends: Friend[]
  pending?: boolean
}>()

const emit = defineEmits<{
  close: []
  invite: [id: string]
}>()

const query = ref('')
const selectedId = ref<string | null>(null)

watch(
  () => props.open,
  (open) => {
    if (!open) {
      query.value = ''
      selectedId.value = null
    }
  },
)

watch(
  () => props.friends,
  (friends) => {
    if (
      selectedId.value
      && !friends.some(item => item.friend_id === selectedId.value)
    ) {
      selectedId.value = null
    }
  },
)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) {
    return props.friends
  }
  return props.friends.filter(item =>
    item.username.toLowerCase().includes(q)
    || item.email.toLowerCase().includes(q),
  )
})

function isSelected(id: string) {
  return selectedId.value === id
}

function toggle(id: string) {
  // 再次点击当前好友则取消选择；点击其他好友则切换选择
  selectedId.value = selectedId.value === id ? null : id
}

function submit() {
  if (!selectedId.value || props.pending) {
    return
  }
  emit('invite', selectedId.value)
}
</script>

<template>
  <KgModal
    :open="open"
    :title="$t('chat.invite')"
    @close="emit('close')"
  >
    <div class="picker">
      <input
        v-model="query"
        class="input"
        :placeholder="$t('friends.searchPlaceholder')"
      >
      <div class="list">
        <p
          v-if="!filtered.length"
          class="muted empty"
        >
          {{ $t('chat.inviteEmpty') }}
        </p>
        <button
          v-for="friend in filtered"
          :key="friend.friend_id"
          type="button"
          class="item"
          :class="{ selected: isSelected(friend.friend_id) }"
          :aria-pressed="isSelected(friend.friend_id)"
          @click="toggle(friend.friend_id)"
        >
          <Avatar
            :name="friend.username"
            :seed="friend.friend_id"
            :size="40"
          />
          <div class="meta">
            <strong
              class="truncate"
              :title="friend.username"
            >{{ friend.username }}</strong>
            <span
              class="muted truncate"
              :title="friend.email"
            >{{ friend.email }}</span>
          </div>
          <span
            class="mark"
            :class="{ on: isSelected(friend.friend_id) }"
          />
        </button>
      </div>
      <div class="modal-actions">
        <button
          class="btn btn-secondary"
          type="button"
          @click="emit('close')"
        >
          {{ $t('app.cancel') }}
        </button>
        <button
          class="btn btn-primary"
          type="button"
          :disabled="pending || !selectedId"
          @click="submit"
        >
          {{ $t('chat.invite') }}
        </button>
      </div>
    </div>
  </KgModal>
</template>

<style scoped>
.picker {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: min(360px, 50vh);
  overflow: auto;
  margin: 0 -8px;
  padding: 4px 8px;
}

.item {
  display: flex;
  gap: 10px;
  align-items: center;
  width: 100%;
  padding: 10px;
  border-radius: var(--radius-lg);
  color: inherit;
  text-align: left;
}

.item.selected,
.item:hover {
  background: var(--bg-active);
}

.meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.mark {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 1.5px solid var(--color-border);
}

.mark.on {
  border-color: var(--color-primary);
  background: var(--color-primary);
}

.mark.on::after {
  content: '';
  display: block;
  width: 5px;
  height: 9px;
  margin: 2px auto 0;
  border: solid #fff;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.empty {
  padding: 16px 10px;
  text-align: center;
}
</style>
