<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { isMobile } = useIsMobile()
</script>

<template>
  <div
    v-if="!isMobile"
    class="empty"
  >
    <ShortcutGrid />
  </div>
</template>

<style scoped>
.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
}
</style>
