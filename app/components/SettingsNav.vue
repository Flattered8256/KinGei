<script setup lang="ts">
const route = useRoute()
const { logout } = useAuth()
const { isMobile } = useIsMobile()

const items = [
  { to: '/settings/account', key: 'account' },
  { to: '/settings/language', key: 'language' },
  { to: '/settings/appearance', key: 'appearance' },
]

function active(to: string) {
  return route.path === to
}

async function signOut() {
  logout()
  await navigateTo('/login')
}
</script>

<template>
  <nav class="nav">
    <header>
      <NuxtLink
        v-if="!isMobile"
        to="/chat"
        class="icon-btn"
        :aria-label="$t('app.back')"
      >
        <AppIcon name="back" :size="18" />
      </NuxtLink>
      <h1>{{ $t('settings.title') }}</h1>
    </header>
    <div class="items">
      <NuxtLink
        v-for="item in items"
        :key="item.key"
        :to="item.to"
        class="item"
        :class="{ active: active(item.to) }"
      >
        {{ $t(`settings.${item.key}`) }}
      </NuxtLink>
    </div>
    <button
      type="button"
      class="logout"
      @click="signOut"
    >
      {{ $t('settings.logout') }}
    </button>
  </nav>
</template>

<style scoped>
.nav {
  display: flex;
  flex-direction: column;
  width: var(--settings-nav-width);
  min-width: var(--settings-nav-width);
  height: 100%;
  padding: 12px 12px 16px;
}

header {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 12px;
}

h1 {
  font-size: 18px;
}

.items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.item {
  padding: 10px 12px;
  border-radius: var(--radius-md);
  color: var(--color-text);
}

.item.active,
.item:hover {
  background: var(--bg-active);
}

.logout {
  margin-top: auto;
  padding: 10px 12px;
  color: var(--color-danger);
  text-align: left;
  font-weight: 600;
}

@media (max-width: 899px) {
  .nav {
    width: 100%;
    min-width: 0;
    background: var(--bg-page);
  }
}
</style>
