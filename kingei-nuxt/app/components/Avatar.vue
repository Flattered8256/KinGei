<script setup lang="ts">
import { uploadImage } from '~/utils/api/file'
import { avatarColor, initials } from '~/utils/display'

const props = withDefaults(defineProps<{
  name?: string | null
  seed?: string | null
  size?: number
  hash?: boolean
  fileId?: number | null
  editable?: boolean
}>(), {
  name: '',
  seed: '',
  size: 40,
  hash: true,
  fileId: null,
  editable: false,
})

const emit = defineEmits<{
  uploaded: [id: number]
}>()

const { t } = useI18n()
const toast = useToast()
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
  <component
    :is="editable ? 'button' : 'span'"
    class="avatar"
    :type="editable ? 'button' : undefined"
    :disabled="editable ? pending : undefined"
    :title="editable ? t('avatar.change') : (props.name || undefined)"
    :style="{
      width: `${props.size}px`,
      height: `${props.size}px`,
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
.avatar {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
  padding: 0;
  border: 0;
  border-radius: 50%;
  color: #fff;
  font-weight: 600;
  line-height: 1;
  user-select: none;
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
