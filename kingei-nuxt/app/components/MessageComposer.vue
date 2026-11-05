<script setup lang="ts">
import {
  isAbortError,
  isImageType,
  uploadFile,
} from '~/utils/api/file'
import { formatFileSize, messagePreviewText } from '~/utils/display'

type PendingAttachment = {
  file: File
  localUrl: string
  uploadedId: number | null
  progress: number
  status: 'uploading' | 'processing' | 'ready'
  abort: () => void
}

const messages = useMessages()
const chats = useChats()
const users = useUserDirectory()
const { isMobile } = useIsMobile()
const toast = useToast()
const { t } = useI18n()
const draft = ref('')
const area = ref<HTMLTextAreaElement | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const pending = ref<PendingAttachment | null>(null)
const dragging = ref(false)
let attachGen = 0
let dragCount = 0

const replyTo = computed(() => messages.replyTo.value)

const replyTitle = computed(() => {
  const target = replyTo.value
  if (!target) {
    return ''
  }
  const chatId = messages.currentId.value
  const chat = chatId == null ? null : chats.chats.value.find(item => item.id === chatId)
  const member = chat?.member.find(item => item.user.id === target.sender_id)?.user
  const name = member?.username ?? users.get(target.sender_id)?.username ?? t('app.unknownUser')
  return t('chat.replyTo', { name })
})

const replyPreview = computed(() => {
  const target = replyTo.value
  if (!target) {
    return ''
  }
  return messagePreviewText(target, {
    image: t('chat.previewImage'),
    video: t('chat.previewVideo'),
    file: t('chat.previewFile'),
    deleted: t('chat.messageDeleted'),
  })
})

const canSend = computed(() => {
  if (messages.sending.value) {
    return false
  }
  if (pending.value && pending.value.status !== 'ready') {
    return false
  }
  return Boolean(draft.value.trim() || pending.value?.uploadedId != null)
})

const statusText = computed(() => {
  const item = pending.value
  if (!item) {
    return ''
  }
  if (item.status === 'uploading') {
    return `${t('chat.uploading')} ${item.progress}%`
  }
  if (item.status === 'processing') {
    return t('chat.processing')
  }
  return formatFileSize(item.file.size)
})

function fit() {
  const el = area.value
  if (!el) {
    return
  }
  el.style.height = '0px'
  el.style.height = `${el.scrollHeight}px`
}

function disposePending() {
  const current = pending.value
  pending.value = null
  current?.abort()
  if (current?.localUrl) {
    URL.revokeObjectURL(current.localUrl)
  }
}

function clearPending() {
  attachGen += 1
  disposePending()
}

function pick() {
  if (messages.sending.value) {
    return
  }
  fileInput.value?.click()
}

function onFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file) {
    void attach(file)
  }
}

async function attach(file: File) {
  if (file.size <= 0) {
    toast.error(t('error.InvalidSize'))
    return
  }

  disposePending()
  const gen = ++attachGen
  const controller = new AbortController()
  const localUrl = URL.createObjectURL(file)
  pending.value = {
    file,
    localUrl,
    uploadedId: null,
    progress: 0,
    status: 'uploading',
    abort: () => controller.abort(),
  }

  try {
    const uploaded = await uploadFile(file, {
      signal: controller.signal,
      onProgress: (percent) => {
        if (gen === attachGen && pending.value) {
          pending.value.progress = percent
        }
      },
      onPhase: (phase) => {
        if (gen === attachGen && pending.value) {
          pending.value.status = phase
        }
      },
    })
    if (gen !== attachGen || !pending.value) {
      return
    }
    pending.value.uploadedId = uploaded.id
    pending.value.progress = 100
    pending.value.status = 'ready'
  }
  catch (err) {
    if (gen !== attachGen || isAbortError(err)) {
      return
    }
    toast.error(err)
    if (gen === attachGen) {
      disposePending()
    }
  }
}

async function submit() {
  const text = draft.value
  const fileId = pending.value?.status === 'ready' ? pending.value.uploadedId ?? undefined : undefined
  if (messages.sending.value || (!text.trim() && fileId == null)) {
    return
  }
  if (pending.value && pending.value.status !== 'ready') {
    return
  }

  draft.value = ''
  messages.saveDraft(messages.currentId.value, '')
  const held = pending.value
  pending.value = null
  try {
    await messages.send(text, fileId)
    if (held?.localUrl) {
      URL.revokeObjectURL(held.localUrl)
    }
  }
  catch (err) {
    draft.value = text
    messages.saveDraft(messages.currentId.value, text)
    pending.value = held
    toast.error(err)
  }
  await nextTick()
  fit()
  if (!isMobile.value) {
    area.value?.focus()
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    void submit()
  }
}

function onPaste(event: ClipboardEvent) {
  const file = event.clipboardData?.files[0]
  if (!file) {
    return
  }
  event.preventDefault()
  void attach(file)
}

function onDragEnter(event: DragEvent) {
  if (!event.dataTransfer?.types.includes('Files')) {
    return
  }
  event.preventDefault()
  dragCount += 1
  dragging.value = true
}

function onDragOver(event: DragEvent) {
  if (!event.dataTransfer?.types.includes('Files')) {
    return
  }
  event.preventDefault()
  event.dataTransfer.dropEffect = 'copy'
}

function onDragLeave(event: DragEvent) {
  if (!event.dataTransfer?.types.includes('Files')) {
    return
  }
  dragCount -= 1
  if (dragCount <= 0) {
    dragCount = 0
    dragging.value = false
  }
}

function onDrop(event: DragEvent) {
  event.preventDefault()
  dragCount = 0
  dragging.value = false
  const file = event.dataTransfer?.files[0]
  if (file) {
    void attach(file)
  }
}

function onQuoteSelect() {
  const target = replyTo.value
  if (target) {
    messages.reveal(target.id)
  }
}

watch(draft, () => {
  nextTick(fit)
})

watch(replyTo, (target) => {
  if (!target) {
    return
  }
  nextTick(() => {
    area.value?.focus()
  })
})

watch(
  () => messages.currentId.value,
  (id, prev) => {
    if (prev != null) {
      messages.saveDraft(prev, draft.value)
    }
    draft.value = messages.takeDraft(id)
    clearPending()
    nextTick(fit)
  },
  { immediate: true, flush: 'sync' },
)

onMounted(fit)
onUnmounted(() => {
  messages.saveDraft(messages.currentId.value, draft.value)
  clearPending()
})
</script>

<template>
  <form
    class="composer"
    :class="{ dragging }"
    @submit.prevent="submit"
    @dragenter="onDragEnter"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <div class="row">
      <button
        type="button"
        class="tool"
        :disabled="messages.sending.value"
        :aria-label="$t('chat.attach')"
        :title="$t('chat.attach')"
        @click="pick"
      >
        <AppIcon name="paperclip" :size="20" />
      </button>
      <div class="box">
        <div
          v-if="replyTo"
          class="quote-chip"
        >
          <div
            class="quote-pick"
            role="button"
            tabindex="0"
            @click="onQuoteSelect"
            @keydown.enter.prevent="onQuoteSelect"
            @keydown.space.prevent="onQuoteSelect"
          >
            <span class="quote-accent" />
            <span class="quote-name truncate">{{ replyTitle }}</span>
            <span class="quote-text truncate">{{ replyPreview }}</span>
          </div>
          <button
            type="button"
            class="quote-close"
            :aria-label="$t('chat.cancelReply')"
            :title="$t('chat.cancelReply')"
            @click="messages.clearReply()"
          >
            <AppIcon name="close" :size="12" />
          </button>
        </div>
        <div
          v-if="pending"
          class="quote-chip"
        >
          <div class="attach-body">
            <img
              v-if="isImageType(pending.file.type)"
              class="thumb"
              :src="pending.localUrl"
              alt=""
            >
            <span
              v-else
              class="thumb icon"
            >
              <AppIcon name="file" :size="16" />
            </span>
            <div class="attach-copy">
              <span
                class="quote-name truncate"
                :title="pending.file.name"
              >{{ pending.file.name }}</span>
              <span class="quote-text truncate">{{ statusText }}</span>
              <div
                v-if="pending.status !== 'ready'"
                class="bar"
              >
                <i :style="{ width: `${pending.progress}%` }" />
              </div>
            </div>
          </div>
          <button
            type="button"
            class="quote-close"
            :aria-label="$t('chat.removeAttachment')"
            :title="$t('chat.removeAttachment')"
            @click="clearPending"
          >
            <AppIcon name="close" :size="12" />
          </button>
        </div>
        <textarea
          ref="area"
          v-model="draft"
          class="area"
          rows="1"
          :placeholder="$t('chat.inputPlaceholder')"
          @keydown="onKeydown"
          @paste="onPaste"
        />
      </div>
      <button
        class="send"
        type="submit"
        :disabled="!canSend"
        :aria-label="$t('app.send')"
      >
        <AppIcon name="send" :size="16" />
      </button>
    </div>
    <input
      ref="fileInput"
      class="file"
      type="file"
      @change="onFile"
    >
  </form>
</template>

<style scoped>
.composer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--color-border);
}

.composer.dragging .box {
  border-color: var(--color-primary);
}

.row {
  display: flex;
  align-items: flex-end;
  gap: 8px;
}

.box {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  min-height: 40px;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--bg-input);
}

.box:focus-within {
  border-color: var(--color-primary);
}

.quote-chip {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 20px;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-md);
  background: var(--bg-muted);
}

[data-theme='dark'] .quote-chip {
  background: var(--bg-panel);
}

.quote-pick {
  display: grid;
  grid-template-columns: 3px minmax(0, 1fr);
  grid-template-rows: auto auto;
  column-gap: 8px;
  row-gap: 2px;
  align-items: center;
  min-width: 0;
  cursor: pointer;
}

.quote-accent {
  grid-column: 1;
  grid-row: 1 / span 2;
  width: 3px;
  height: 28px;
  border-radius: 2px;
  background: var(--color-primary);
}

.quote-name,
.quote-text {
  grid-column: 2;
  min-width: 0;
  font-size: 12px;
}

.quote-name {
  color: var(--color-primary);
  font-weight: 600;
}

.quote-text {
  color: var(--color-muted);
}

.quote-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--color-border);
  color: var(--color-muted);
}

[data-theme='dark'] .quote-close {
  background: var(--bg-active);
}

.tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  color: var(--color-section);
}

.tool:hover:not(:disabled) {
  color: var(--color-text);
}

.tool:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.area {
  display: block;
  width: 100%;
  min-height: 24px;
  max-height: 120px;
  padding: 0;
  border: 0;
  background: transparent;
  resize: none;
  overflow-y: auto;
}

.area:focus {
  outline: none;
}

.area::placeholder {
  color: var(--color-placeholder);
}

.send {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: 50%;
  background: var(--color-primary);
  color: #fff;
}

.send:disabled {
  opacity: 0.45;
}

.attach-body {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.thumb {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  object-fit: cover;
  border-radius: 6px;
}

.thumb.icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-panel);
  color: var(--color-primary);
}

[data-theme='dark'] .thumb.icon {
  background: var(--bg-muted);
}

.attach-copy {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bar {
  height: 3px;
  margin-top: 2px;
  overflow: hidden;
  border-radius: 2px;
  background: var(--color-border);
}

.bar i {
  display: block;
  height: 100%;
  background: var(--color-primary);
}

.file {
  display: none;
}
</style>
