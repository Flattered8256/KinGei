<script setup lang="ts">
const route = useRoute()
const chats = useChats()
const friends = useFriends()

const items = computed(() => [
  { name: 'home' as const, to: '/home', match: '/home', badge: 0 },
  { name: 'chat' as const, to: '/chat', match: '/chat', badge: chats.unreadTotal.value },
  { name: 'friends' as const, to: '/friends', match: '/friends', badge: friends.incomingCount.value },
  { name: 'settings' as const, to: '/settings', match: '/settings', badge: 0 },
])

function active(match: string) {
  return route.path === match || route.path.startsWith(`${match}/`)
}
</script>

<template>
  <nav class="tabbar">
    <NuxtLink
      v-for="item in items"
      :key="item.name"
      :to="item.to"
      class="tab"
      :class="{ active: active(item.match) }"
    >
      <UnreadBadge
        v-if="item.name === 'chat' || item.name === 'friends'"
        :value="item.badge"
      >
        <AppIcon :name="item.name" :size="22" />
      </UnreadBadge>
      <AppIcon
        v-else
        :name="item.name"
        :size="22"
      />
      <span>{{ $t(`nav.${item.name}`) }}</span>
    </NuxtLink>
  </nav>
</template>

<style scoped>
.tabbar {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  height: var(--tabbar-height);
  background: var(--bg-panel);
  border-top: 1px solid var(--color-border);
}

.tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: var(--color-muted);
  font-size: 11px;
}

.tab.active {
  color: var(--color-primary);
}
</style>
