<script setup lang="ts">
const { open } = useAppActions()
const { canAddFriend, canCreateGroup } = useInstance()

const empty = computed(() => !canAddFriend.value && !canCreateGroup.value)
</script>

<template>
  <p
    v-if="empty"
    class="muted empty"
  >
    {{ $t('chat.shortcutsEmpty') }}
  </p>
  <div
    v-else
    class="shortcuts"
    :class="{ single: !canAddFriend || !canCreateGroup }"
  >
    <button
      v-if="canCreateGroup"
      type="button"
      class="card"
      @click="open('group')"
    >
      <AppIcon name="plus" :size="28" />
      <span>{{ $t('chat.createGroupHint') }}</span>
    </button>
    <button
      v-if="canAddFriend"
      type="button"
      class="card"
      @click="open('friend')"
    >
      <AppIcon name="friends" :size="28" />
      <span>{{ $t('chat.addFriendHint') }}</span>
    </button>
  </div>
</template>

<style scoped>
.shortcuts {
  display: grid;
  grid-template-columns: repeat(2, minmax(160px, 220px));
  gap: 16px;
  justify-content: center;
  padding: 24px;
}

.shortcuts.single {
  grid-template-columns: minmax(160px, 220px);
}

.empty {
  padding: 32px 16px;
  text-align: center;
}

.card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 140px;
  padding: 20px;
  border-radius: var(--radius-xl);
  background: var(--bg-muted);
  color: var(--color-text);
  text-align: center;
}

.card :deep(svg) {
  color: var(--color-primary);
}

.card:hover {
  background: var(--bg-active);
}

@media (max-width: 899px) {
  .shortcuts {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    padding: 32px 16px;
  }

  .shortcuts.single {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
