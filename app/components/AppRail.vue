<script setup lang="ts">
const { user } = useAuth()
const route = useRoute()
const chats = useChats()
const friends = useFriends()

const items = computed(() => [
  {
    name: 'chat' as const,
    to: '/chat',
    active: route.path.startsWith('/chat'),
    badge: chats.unreadTotal.value,
  },
  {
    name: 'friends' as const,
    to: '/friends',
    active: route.path.startsWith('/friends'),
    badge: friends.incomingCount.value,
  },
])

const settingsActive = computed(() => route.path.startsWith('/settings'))
</script>

<template>
  <aside class="rail">
    <NuxtLink
      to="/settings/account"
      class="rail-avatar"
    >
      <Avatar
        :name="user?.username"
        :seed="user?.id"
        :size="40"
      />
    </NuxtLink>
    <nav class="rail-nav">
      <NuxtLink
        v-for="item in items"
        :key="item.name"
        :to="item.to"
        class="icon-btn"
        :class="{ active: item.active }"
        :title="$t(`nav.${item.name}`)"
      >
        <UnreadBadge :value="item.badge">
          <AppIcon :name="item.name" />
        </UnreadBadge>
      </NuxtLink>
    </nav>
    <NuxtLink
      to="/settings"
      class="icon-btn rail-settings"
      :class="{ active: settingsActive }"
      :title="$t('nav.settings')"
    >
      <AppIcon name="settings" />
    </NuxtLink>
  </aside>
</template>

<style scoped>
.rail {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: var(--rail-width);
  padding: 16px 0;
  gap: 8px;
}

.rail-avatar {
  display: flex;
  margin-bottom: 8px;
}

.rail-nav {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rail-settings {
  margin-top: auto;
}
</style>
