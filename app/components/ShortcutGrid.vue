<script setup lang="ts">
const { open } = useAppActions()
</script>

<template>
  <div class="shortcuts">
    <button
      type="button"
      class="card"
      @click="open('group')"
    >
      <AppIcon name="plus" :size="28" />
      <span>{{ $t('chat.createGroupHint') }}</span>
    </button>
    <button
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
}
</style>
