<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})
</script>

<template>
  <div class="blank" />
</template>

<style scoped>
.blank {
  flex: 1;
}
</style>
