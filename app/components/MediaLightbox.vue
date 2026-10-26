<script setup lang="ts">
import type { FileResponse } from '~/types/api'
import { refreshSession } from '~/utils/api/http'
import { attachmentKind } from '~/utils/display'

const props = defineProps<{
  file: FileResponse | null
}>()

const emit = defineEmits<{
  close: []
}>()

const kind = computed(() => (props.file ? attachmentKind(props.file) : 'file'))
const { src, reload } = useFileSrc(() => props.file?.id ?? null)
const { src: posterSrc } = useFileSrc(() => props.file?.media?.thumbnail_id ?? null)
const videoEl = ref<HTMLVideoElement | null>(null)
const retried = ref(false)

function close() {
  emit('close')
}

async function onImageError() {
  if (retried.value) {
    return
  }
  retried.value = true
  await reload()
}

async function onVideoError() {
  if (retried.value) {
    return
  }
  retried.value = true
  try {
    await refreshSession()
  }
  catch {
    return
  }
  videoEl.value?.load()
}

function onBackdrop(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    close()
  }
}

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    close()
  }
}

watch(
  () => props.file,
  (file, prev) => {
    if (file?.id !== prev?.id) {
      retried.value = false
    }
    if (!import.meta.client) {
      return
    }
    if (file) {
      window.addEventListener('keydown', onKey)
    }
    else {
      window.removeEventListener('keydown', onKey)
    }
  },
)

onUnmounted(() => {
  if (import.meta.client) {
    window.removeEventListener('keydown', onKey)
  }
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="file"
      class="backdrop"
      role="dialog"
      aria-modal="true"
      @mousedown="onBackdrop"
    >
      <button
        type="button"
        class="icon-btn close"
        :aria-label="$t('app.close')"
        @click="close"
      >
        <AppIcon name="close" :size="20" />
      </button>
      <p
        v-if="!src"
        class="loading"
      >
        {{ $t('app.loading') }}
      </p>
      <img
        v-else-if="kind === 'image'"
        :src="src"
        :alt="file.name"
        class="stage"
        @error="onImageError"
      >
      <video
        v-else-if="kind === 'video'"
        ref="videoEl"
        class="stage"
        :src="src"
        :poster="posterSrc ?? undefined"
        controls
        autoplay
        @error="onVideoError"
      />
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px 16px 16px;
  background: rgb(0 0 0 / 82%);
}

.close {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 1;
  color: #fff;
}

.close:hover {
  background: rgb(255 255 255 / 12%);
  color: #fff;
}

.stage {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  background: #000;
}

.loading {
  color: #fff;
}
</style>
