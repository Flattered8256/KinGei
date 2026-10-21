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
    class="empty-state"
  >
    <p>{{ $t('friends.empty') }}</p>
  </div>
</template>
