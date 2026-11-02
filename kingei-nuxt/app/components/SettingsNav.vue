<script setup lang="ts">
const route = useRoute()
const { logout } = useAuth()
const { isMobile } = useIsMobile()
const { isAdmin } = useInstance()

const general = [
  { to: '/settings/account', key: 'account' },
  { to: '/settings/language', key: 'language' },
  { to: '/settings/appearance', key: 'appearance' },
]

const admin = [
  { to: '/settings/site', key: 'site' },
  { to: '/settings/users', key: 'users' },
]

function active(to: string) {
  return route.path === to
}

async function signOut() {
  await logout()
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
      <p class="group">
        {{ $t('settings.general') }}
      </p>
      <NuxtLink
        v-for="item in general"
        :key="item.key"
        :to="item.to"
        class="item"
        :class="{ active: active(item.to) }"
      >
        {{ $t(`settings.${item.key}`) }}
      </NuxtLink>
      <template v-if="isAdmin">
        <p class="group admin">
          {{ $t('settings.admin') }}
        </p>
        <NuxtLink
          v-for="item in admin"
          :key="item.key"
          :to="item.to"
          class="item"
          :class="{ active: active(item.to) }"
        >
          {{ $t(`settings.${item.key}`) }}
        </NuxtLink>
      </template>
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

.group {
  padding: 8px 10px 2px;
  color: var(--color-section);
  font-size: 12px;
  font-weight: 700;
}

.group.admin {
  margin-top: 8px;
}

.item {
  padding: 8px 10px;
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-weight: 600;
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
  font-weight: 700;
}

@media (max-width: 899px) {
  .nav {
    width: 100%;
    min-width: 0;
    background: var(--bg-page);
  }

  h1 {
    font-size: 22px;
  }

  .item {
    padding: 10px 12px;
    font-size: 16px;
  }
}
</style>
