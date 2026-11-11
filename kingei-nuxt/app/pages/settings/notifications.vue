<script setup lang="ts">
import { ElSwitch } from 'element-plus'
import 'element-plus/es/components/switch/style/css'

definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { isMobile } = useIsMobile()
const { armed, denied, supported, setArmed } = useSystemNotify()

function onToggle(val: string | number | boolean) {
  void setArmed(Boolean(val))
}
</script>

<template>
  <section class="page">
    <header class="top">
      <NuxtLink
        v-if="isMobile"
        to="/settings"
        class="icon-btn"
        :aria-label="$t('app.back')"
      >
        <AppIcon name="back" />
      </NuxtLink>
      <h1>{{ $t('settings.notifications') }}</h1>
    </header>
    <div class="box">
      <div class="row">
        <div class="copy">
          <strong>{{ $t('settings.notifications') }}</strong>
          <span class="muted">{{ $t('settings.notificationsHint') }}</span>
          <span
            v-if="!supported"
            class="muted warn"
          >{{ $t('settings.notificationsUnsupported') }}</span>
          <span
            v-else-if="denied"
            class="muted warn"
          >{{ $t('settings.notificationsDenied') }}</span>
        </div>
        <el-switch
          class="toggle"
          :model-value="armed"
          :disabled="!supported || denied"
          :aria-label="$t('settings.notifications')"
          @update:model-value="onToggle"
        />
      </div>
    </div>
  </section>
</template>

<style scoped>
.page {
  padding: 8px 24px 32px;
}

.top {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 52px;
}

h1 {
  font-size: 22px;
}

.box {
  max-width: 560px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
}

.copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.copy strong {
  font-size: 14px;
}

.warn {
  color: var(--color-danger);
}

.toggle {
  flex-shrink: 0;
  --el-switch-on-color: var(--color-primary);
  --el-switch-off-color: var(--color-border);
}

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }
}
</style>
