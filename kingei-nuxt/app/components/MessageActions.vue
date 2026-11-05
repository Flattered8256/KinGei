<script setup lang="ts">
withDefaults(defineProps<{
  own?: boolean
}>(), {
  own: false,
})
</script>

<template>
  <div
    class="actions"
    :class="{ own }"
  >
    <slot />
  </div>
</template>

<style scoped>
.actions {
  position: absolute;
  bottom: 0;
  left: calc(100% + 6px);
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--bg-panel) 88%, white);
  box-shadow: 0 1px 2px rgb(15 23 42 / 6%);
}

.actions.own {
  left: auto;
  right: calc(100% + 6px);
}

.actions :slotted(button) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  color: var(--color-muted);
}

.actions :slotted(button:hover:not(:disabled)) {
  background: var(--bg-hover);
  color: var(--color-text);
}

.actions :slotted(button.danger:hover:not(:disabled)) {
  background: var(--color-danger-bg);
  color: var(--color-danger);
}

.actions :slotted(button:disabled) {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
