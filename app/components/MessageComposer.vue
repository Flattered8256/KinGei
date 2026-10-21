<script setup lang="ts">
const messages = useMessages()
const toast = useToast()
const draft = ref('')
const area = ref<HTMLTextAreaElement | null>(null)

function fit() {
  const el = area.value
  if (!el) {
    return
  }
  el.style.height = '0px'
  el.style.height = `${el.scrollHeight}px`
}

async function submit() {
  const text = draft.value
  if (!text.trim() || messages.sending.value) {
    return
  }
  draft.value = ''
  try {
    await messages.send(text)
  }
  catch (err) {
    draft.value = text
    toast.error(err)
  }
  await nextTick()
  fit()
  area.value?.focus()
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    void submit()
  }
}

watch(draft, () => {
  nextTick(fit)
})

onMounted(fit)
</script>

<template>
  <form
    class="composer"
    @submit.prevent="submit"
  >
    <div class="field-wrap">
      <textarea
        ref="area"
        v-model="draft"
        class="box"
        rows="1"
        :placeholder="$t('chat.inputPlaceholder')"
        @keydown="onKeydown"
      />
    </div>
    <button
      class="send"
      type="submit"
      :disabled="messages.sending.value || !draft.trim()"
      :aria-label="$t('app.send')"
    >
      <AppIcon name="send" :size="18" />
    </button>
  </form>
</template>

<style scoped>
.composer {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 12px 16px 16px;
  border-top: 1px solid var(--color-border);
}

.field-wrap {
  flex: 1;
  min-width: 0;
}

.box {
  display: block;
  width: 100%;
  min-height: 44px;
  max-height: 120px;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 22px;
  background: var(--bg-input);
  resize: none;
  overflow-y: auto;
}

.box:focus {
  outline: none;
  border-color: var(--color-primary);
}

.send {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--color-primary);
  color: #fff;
}

.send:disabled {
  opacity: 0.45;
}
</style>
