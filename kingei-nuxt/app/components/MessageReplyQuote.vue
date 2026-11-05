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
    <span class="accent" />
    <span class="copy">
      <span class="quote-name truncate">{{ title }}</span>
      <span class="quote-text truncate">{{ preview }}</span>
    </span>
  </button>
</template>

<style scoped>
.quote {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 8px;
  width: fit-content;
  max-width: 100%;
  overflow: hidden;
  padding: 6px 10px 6px 8px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--bg-quote);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.accent {
  width: 2px;
  height: 26px;
  flex-shrink: 0;
  border-radius: 2px;
  background: var(--color-primary);
}

.copy {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  min-width: 0;
}

.quote-name,
.quote-text {
  max-width: 100%;
  font-size: 12px;
  line-height: 1.3;
}

.quote-name {
  color: var(--color-primary);
  font-weight: 600;
}

.quote-text {
  color: var(--color-quote);
}
</style>
