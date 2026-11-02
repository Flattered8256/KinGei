<script setup lang="ts">
definePageMeta({
  middleware: [
    'auth',
    () => navigateTo('/chat'),
  ],
})
</script>

<template>
  <div />
</template>
