<script setup lang="ts">
const route = useRoute()
const { isMobile } = useIsMobile()

const { ready } = useAppSync()

const hideChrome = computed(() => {
  if (!isMobile.value) {
    return false
  }
  return /^\/chat\/[^/]+$/.test(route.path)
    || /^\/friends\/[^/]+$/.test(route.path)
    || /^\/settings\/.+/.test(route.path)
})

const showRail = computed(() => !isMobile.value && !route.path.startsWith('/settings'))
const showTabbar = computed(() => isMobile.value && !hideChrome.value)
const padded = computed(() => !isMobile.value)
</script>

<template>
  <div
    class="app-shell"
    :class="{ mobile: isMobile, 'has-tabbar': showTabbar }"
  >
    <AppRail v-if="showRail" />
    <div
      class="app-stage"
      :class="{ padded, 'with-rail': showRail }"
    >
      <slot />
    </div>
    <AppTabbar v-if="showTabbar" />
    <AppModals />
    <KgOverlay
      v-if="!ready"
      global
    />
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  display: flex;
  height: 100%;
  overflow: hidden;
  background: var(--bg-page);
}

.app-stage {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.app-stage.padded {
  padding: 8px;
}

.app-stage.padded.with-rail {
  padding-left: 0;
}

.app-shell.mobile {
  flex-direction: column;
}

.app-shell.mobile .app-stage {
  padding: 0;
}

.app-shell.has-tabbar .app-stage {
  padding-bottom: 0;
}
</style>
