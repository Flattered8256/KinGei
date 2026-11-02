<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const { isMobile } = useIsMobile()
const isSettingsHome = computed(() => route.path.replace(/\/+$/, '') === '/settings')
const showNav = computed(() => !isMobile.value || isSettingsHome.value)
</script>

<template>
  <div
    class="settings"
    :class="{ mobile: isMobile }"
  >
    <SettingsNav v-show="showNav" />
    <div
      v-show="!isMobile || !isSettingsHome"
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
