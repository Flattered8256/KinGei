<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
  key: 'chat',
})

const route = useRoute()
const { isMobile } = useIsMobile()
const messages = useMessages()
const showList = computed(() => !isMobile.value || !route.params.id)

watch(
  () => route.params.id,
  (id, prev) => {
    if (prev && !id) {
      messages.close(Number(prev))
    }
  },
)

onBeforeUnmount(() => {
  messages.close()
})
</script>

<template>
  <div class="panel-card chat-page">
    <SessionList v-show="showList" />
    <div
      v-show="!isMobile || Boolean(route.params.id)"
      class="body"
    >
      <NuxtPage :page-key="route.params.id ? 'chat-room' : 'chat-empty'" />
    </div>
  </div>
</template>

<style scoped>
.chat-page {
  height: 100%;
}

.body {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

@media (max-width: 899px) {
  .chat-page {
    display: flex;
  }

  .body {
    width: 100%;
  }
}
</style>
