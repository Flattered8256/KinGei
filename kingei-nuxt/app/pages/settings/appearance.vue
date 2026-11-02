<script setup lang="ts">
import type { ThemePreference } from '~/composables/useTheme'

definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { preference, setPreference } = useTheme()
const { isMobile } = useIsMobile()

const options: Array<{ value: ThemePreference, labelKey: string }> = [
  { value: 'light', labelKey: 'settings.themeLight' },
  { value: 'dark', labelKey: 'settings.themeDark' },
  { value: 'system', labelKey: 'settings.themeSystem' },
]
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
      <h1>{{ $t('settings.appearance') }}</h1>
    </header>
    <div class="choices">
      <button
        v-for="option in options"
        :key="option.value"
        type="button"
        class="choice"
        :class="{ active: preference === option.value }"
        @click="setPreference(option.value)"
      >
        {{ $t(option.labelKey) }}
      </button>
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

.choices {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 420px;
  margin-top: 12px;
}

.choice {
  padding: 12px 14px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  text-align: left;
}

.choice.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--bg-muted);
}

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }
}
</style>
