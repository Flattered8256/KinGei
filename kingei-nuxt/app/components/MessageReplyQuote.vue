<script setup lang="ts">
import type { Message } from '~/types/api'
import { messagePreviewText } from '~/utils/display'

const props = defineProps<{
  title: string
  message: Message
}>()

const emit = defineEmits<{
  select: []
}>()

const { t } = useI18n()

const preview = computed(() => messagePreviewText(props.message, {
  image: t('chat.previewImage'),
  video: t('chat.previewVideo'),
  file: t('chat.previewFile'),
  deleted: t('chat.messageDeleted'),
}))
</script>

<template>
  <button
    type="button"
    class="quote"
    @click.stop="emit('select')"
  >
    <span class="quote-name truncate">{{ title }}</span>
    <span class="quote-text truncate">{{ preview }}</span>
  </button>
</template>

<style scoped>
.quote {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  align-self: stretch;
  min-width: 120px;
  max-width: 100%;
  overflow: hidden;
  padding: 6px 10px;
  border: 0;
  border-left: 3px solid var(--color-primary);
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  background: color-mix(in srgb, var(--color-text) 6%, transparent);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.quote-name {
  max-width: 100%;
  color: var(--color-primary);
  font-size: 12px;
  font-weight: 600;
}

.quote-text {
  max-width: 100%;
  color: var(--color-muted);
  font-size: 12px;
}
</style>
