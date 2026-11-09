<script setup lang="ts">
import { avatarColor, initials } from '~/utils/display'

const MIN_PRESENCE_SIZE = 24

const props = withDefaults(defineProps<{
  name?: string | null
  seed?: string | null
  size?: number
  hash?: boolean
  fileId?: number | null
  editable?: boolean
  userId?: string | null
}>(), {
  name: '',
  seed: '',
  size: 40,
  hash: true,
  fileId: null,
  editable: false,
  userId: null,
})

const emit = defineEmits<{
  uploaded: [id: number]
}>()

const { t } = useI18n()
const toast = useToast()
const presence = usePresence()
const { uploadImage } = useFileUpload()
const { src, reload } = useFileSrc(() => props.fileId)
const broken = ref(false)
const retried = ref(false)
const pending = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

watch(() => props.fileId, () => {
  broken.value = false
  retried.value = false
})

const showImage = computed(() => Boolean(src.value) && !broken.value)
const letter = computed(() => initials(props.name))
const background = computed(() =>
  props.hash ? avatarColor(props.seed || props.name || '') : '#22ccee',
)
const fontSize = computed(() => Math.round(props.size * 0.4))
const maskSize = computed(() => Math.max(10, Math.round(props.size * 0.2)))
const dotSize = computed(() => Math.min(14, Math.max(8, Math.round(props.size * 0.25))))
const showDot = computed(() =>
  Boolean(props.userId) && props.size >= MIN_PRESENCE_SIZE && presence.isOnline(props.userId),
)

function onError() {
  if (!retried.value) {
    retried.value = true
    reload()
    return
  }
  broken.value = true
}

function pick() {
  if (!props.editable || pending.value) {
    return
  }
  fileInput.value?.click()
}

async function onFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) {
    return
  }
  pending.value = true
  try {
    const ready = await uploadImage(file)
    emit('uploaded', ready.id)
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pending.value = false
  }
}
</script>

<template>
  <span
    class="avatar-wrap"
    :style="{
      width: `${props.size}px`,
      height: `${props.size}px`,
    }"
    :aria-label="showDot ? t('presence.online') : undefined"
  >
    <component
      :is="editable ? 'button' : 'span'"
      class="avatar"
      :type="editable ? 'button' : undefined"
      :disabled="editable ? pending : undefined"
      :title="editable ? t('avatar.change') : (props.name || undefined)"
      :style="{
        background: showImage ? 'transparent' : background,
        fontSize: `${fontSize}px`,
      }"
      @click="editable ? pick() : undefined"
    >
      <img
        v-if="showImage && src"
        :src="src"
        alt=""
        @error="onError"
      >
      <template v-else>
        {{ letter }}
      </template>
      <span
        v-if="editable"
        class="mask"
        :style="{ fontSize: `${maskSize}px` }"
      >{{ t('avatar.change') }}</span>
    </component>
    <span
      v-if="showDot"
      class="presence"
      :style="{
        width: `${dotSize}px`,
        height: `${dotSize}px`,
      }"
      aria-hidden="true"
    />
  </span>
  <input
    v-if="editable"
    ref="fileInput"
    class="file"
    type="file"
    accept="image/jpeg,image/png,image/webp,image/gif"
    @change="onFile"
  >
</template>

<style scoped>
.avatar-wrap {
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
}

.avatar {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
  width: 100%;
  height: 100%;
  padding: 0;
  border: 0;
  border-radius: 50%;
  color: #fff;
  font-weight: 600;
  line-height: 1;
  user-select: none;
}

.presence {
  position: absolute;
  right: 0;
  bottom: 0;
  border-radius: 50%;
  background: var(--color-online);
  box-shadow: 0 0 0 2px var(--bg-panel);
  pointer-events: none;
}

button.avatar {
  cursor: pointer;
}

button.avatar:disabled {
  cursor: wait;
}

img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mask {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(0 0 0 / 45%);
  color: #fff;
  font-weight: 600;
  opacity: 0;
  transition: opacity 0.15s;
}

button.avatar:hover .mask,
button.avatar:focus-visible .mask {
  opacity: 1;
}

.file {
  display: none;
}
</style>
