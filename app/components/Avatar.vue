<script setup lang="ts">
import { avatarColor, initials } from '~/utils/display'

const props = withDefaults(defineProps<{
  name?: string | null
  seed?: string | null
  size?: number
  hash?: boolean
}>(), {
  name: '',
  seed: '',
  size: 40,
  hash: true,
})

const letter = computed(() => initials(props.name))
const background = computed(() =>
  props.hash ? avatarColor(props.seed || props.name || '') : '#22ccee',
)
const fontSize = computed(() => Math.round(props.size * 0.4))
</script>

<template>
  <span
    class="avatar"
    :style="{
      width: `${props.size}px`,
      height: `${props.size}px`,
      background: background,
      fontSize: `${fontSize}px`,
    }"
    :title="props.name || undefined"
  >
    {{ letter }}
  </span>
</template>

<style scoped>
.avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 50%;
  color: #fff;
  font-weight: 600;
  line-height: 1;
  user-select: none;
}
</style>
