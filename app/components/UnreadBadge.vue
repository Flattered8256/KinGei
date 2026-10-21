<script setup lang="ts">
import { ElBadge } from 'element-plus'
import 'element-plus/es/components/badge/style/css'

defineProps<{
  value: number
}>()
</script>

<template>
  <el-badge
    :value="value"
    :hidden="value <= 0"
    :max="99"
    color="var(--color-unread)"
    class="unread"
  >
    <slot />
  </el-badge>
</template>

<style scoped>
.unread {
  flex-shrink: 0;
  line-height: 1;
}

.unread :deep(.el-badge__content) {
  border: 0;
}
</style>
