<script setup lang="ts">
const props = defineProps<{
  open: boolean
  title: string
}>()

const emit = defineEmits<{
  close: []
}>()

function onBackdrop(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.open"
      class="modal-backdrop"
      @mousedown="onBackdrop"
    >
      <div
        class="modal-card"
        role="dialog"
        aria-modal="true"
      >
        <div class="modal-head">
          <h2 class="modal-title">
            {{ props.title }}
          </h2>
          <button
            type="button"
            class="icon-btn"
            :aria-label="$t('app.close')"
            @click="emit('close')"
          >
            <AppIcon name="close" :size="18" />
          </button>
        </div>
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.modal-title {
  margin-bottom: 12px;
}
</style>
