<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const { isMobile } = useIsMobile()
const showNav = computed(() => !isMobile.value || route.path === '/settings')
</script>

<template>
  <div
    class="settings"
    :class="{ mobile: isMobile }"
  >
    <SettingsNav v-show="showNav" />
    <div
      v-show="!isMobile || route.path !== '/settings'"
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
  background: var(--bg-panel);
  border-radius: var(--radius-xl);
  overflow: auto;
}

.settings.mobile .content {
  border-radius: 0;
}
</style>
