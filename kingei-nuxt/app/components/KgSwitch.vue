<script setup lang="ts">
const props = withDefaults(defineProps<{
  modelValue: boolean
  disabled?: boolean
}>(), {
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function toggle() {
  if (props.disabled) {
    return
  }
  emit('update:modelValue', !props.modelValue)
}
</script>

<template>
  <button
    type="button"
    class="switch"
    :class="{ on: modelValue }"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="knob" />
  </button>
</template>

<style scoped>
.switch {
  position: relative;
  flex-shrink: 0;
  width: 44px;
  height: 24px;
  padding: 2px;
  border-radius: 12px;
  background: var(--color-border);
  transition: background 0.15s ease;
}

.switch.on {
  background: var(--color-primary);
}

.switch:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.knob {
  display: block;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  transform: translateX(0);
  transition: transform 0.15s ease;
}

.switch.on .knob {
  transform: translateX(20px);
}
</style>
