<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const { isMobile } = useIsMobile()
const settingsPath = computed(() => {
  const path = route.path
  return path.length > 1 ? path.replace(/\/+$/, '') : path
})
const atSettingsRoot = computed(() => settingsPath.value === '/settings')
const showNav = computed(() => !isMobile.value || atSettingsRoot.value)
</script>

<template>
  <div
    class="settings"
    :class="{ mobile: isMobile }"
  >
    <SettingsNav v-show="showNav" />
    <div
      v-show="!isMobile || !atSettingsRoot"
      class="content"
    >
      <NuxtPage />
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  height: 100%;
  min-height: 0;
}

.content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-panel);
  border-radius: var(--radius-xl);
  overflow: auto;
}

.settings.mobile .content {
  border-radius: 0;
}
</style>
