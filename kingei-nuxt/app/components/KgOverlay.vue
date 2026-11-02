<script setup lang="ts">
withDefaults(defineProps<{
  global?: boolean
}>(), {
  global: false,
})
</script>

<template>
  <div
    class="overlay"
    :class="{ global }"
    role="status"
    aria-live="polite"
  >
    <KgSpinner :size="32" />
    <span class="sr-only">{{ $t('app.loading') }}</span>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute;
  inset: 0;
  z-index: 4;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  background: color-mix(in srgb, var(--bg-panel) 72%, transparent);
  backdrop-filter: blur(4px);
  pointer-events: auto;
}

.overlay.global {
  z-index: 40;
  background: color-mix(in srgb, var(--bg-page) 72%, transparent);
}
</style>
