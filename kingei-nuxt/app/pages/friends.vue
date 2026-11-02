<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const { isMobile } = useIsMobile()
const showList = computed(() => !isMobile.value || !route.params.id)
</script>

<template>
  <div class="panel-card friends-page">
    <FriendList v-show="showList" />
    <div
      v-show="!isMobile || route.params.id"
      class="body"
    >
      <NuxtPage />
    </div>
  </div>
</template>

<style scoped>
.friends-page {
  height: 100%;
}

.body {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
}
</style>
