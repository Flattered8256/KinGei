<script setup lang="ts">
import type { FileResponse } from '~/types/api'
import { downloadFile } from '~/utils/api/file'
import { attachmentKind, formatDuration, formatFileSize, mediaBox } from '~/utils/display'

const props = withDefaults(defineProps<{
  file: FileResponse
  own?: boolean
}>(), {
  own: false,
})

const emit = defineEmits<{
  open: []
}>()

const { t } = useI18n()
const toast = useToast()
const downloading = ref(false)
const broken = ref(false)
const retried = ref(false)

const kind = computed(() => attachmentKind(props.file))
const previewId = computed(() => {
  if (props.file.media?.thumbnail_id) {
    return props.file.media.thumbnail_id
  }
  if (kind.value === 'image') {
    return props.file.id
  }
  return null
})
const { src, reload } = useFileSrc(previewId)
const duration = computed(() => formatDuration(props.file.media?.duration_ms))
const box = computed(() => {
  const media = props.file.media
  if (!media) {
    return { width: 240, height: 160 }
  }
  return mediaBox(media.width, media.height)
})
const showImage = computed(() => Boolean(src.value) && !broken.value)

watch(() => props.file.id, () => {
  broken.value = false
  retried.value = false
})

function onError() {
  if (!retried.value) {
    retried.value = true
    reload()
    return
  }
  broken.value = true
}

async function download() {
  if (downloading.value) {
    return
  }
  downloading.value = true
  try {
    const result = await downloadFile(props.file.id)
    const link = document.createElement('a')
    link.href = result.download_url
    link.rel = 'noopener'
    link.target = '_blank'
    document.body.append(link)
    link.click()
    link.remove()
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    downloading.value = false
  }
}
</script>

<template>
  <button
    v-if="kind !== 'file'"
    type="button"
    class="media"
    :style="{ width: `${box.width}px`, height: `${box.height}px` }"
    :aria-label="kind === 'video' ? $t('chat.previewVideo') : $t('chat.previewImage')"
    @click="emit('open')"
  >
    <img
      v-if="showImage && src"
      :src="src"
      alt=""
      @error="onError"
    >
    <span
      v-if="kind === 'video'"
      class="play"
    >
      <AppIcon name="play" :size="22" />
    </span>
    <span
      v-if="kind === 'video' && duration"
      class="duration"
    >{{ duration }}</span>
  </button>
  <div
    v-else
    class="card"
    :class="{ own }"
  >
    <span class="file-icon">
      <AppIcon name="file" :size="22" />
    </span>
    <div class="file-meta">
      <span
        class="name truncate"
        :title="file.name"
      >{{ file.name }}</span>
      <span class="size">{{ formatFileSize(file.size) }}</span>
    </div>
    <button
      type="button"
      class="icon-btn download"
      :disabled="downloading"
      :aria-label="t('chat.download')"
      :title="t('chat.download')"
      @click="download"
    >
      <AppIcon name="download" :size="18" />
    </button>
  </div>
</template>

<style scoped>
.media {
  position: relative;
  display: block;
  flex-shrink: 0;
  padding: 0;
  overflow: hidden;
  border-radius: 12px;
  background: var(--bg-muted);
  cursor: pointer;
}

.media img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.play {
  position: absolute;
  inset: 0;
  margin: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: rgb(0 0 0 / 55%);
  color: #fff;
}

.duration {
  position: absolute;
  right: 8px;
  bottom: 8px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgb(0 0 0 / 55%);
  color: #fff;
  font-size: 12px;
  line-height: 1.4;
}

.card {
  display: flex;
  align-items: center;
  gap: 10px;
  width: min(260px, 100%);
  padding: 10px 8px 10px 12px;
  border-radius: 12px;
  background: var(--color-bubble-other);
}

.card.own {
  background: var(--color-bubble-own);
}

.file-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: 8px;
  background: var(--bg-muted);
  color: var(--color-primary);
}

.file-meta {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.name {
  font-weight: 600;
}

.size {
  color: var(--color-muted);
  font-size: 12px;
}

.download {
  width: 36px;
  height: 36px;
  flex-shrink: 0;
}
</style>
