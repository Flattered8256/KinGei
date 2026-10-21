<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { isMobile } = useIsMobile()

watchEffect(() => {
  if (!isMobile.value) {
    void navigateTo('/chat')
  }
})
</script>

<template>
  <div class="home">
    <ShortcutGrid />
  </div>
</template>

<style scoped>
.home {
  height: 100%;
  background: var(--bg-panel);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
