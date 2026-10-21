<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { isMobile } = useIsMobile()

watchEffect(() => {
  if (!isMobile.value) {
    void navigateTo('/settings/account')
  }
})
</script>

<template>
  <div />
</template>
