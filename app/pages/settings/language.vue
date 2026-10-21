<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { locale, setLocale } = useI18n()
const { isMobile } = useIsMobile()

const options = [
  { code: 'zh-CN' as const, labelKey: 'settings.zh' },
  { code: 'en' as const, labelKey: 'settings.en' },
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
      <h1>{{ $t('settings.language') }}</h1>
    </header>
    <div class="choices">
      <button
        v-for="option in options"
        :key="option.code"
        type="button"
        class="choice"
        :class="{ active: locale === option.code }"
        @click="setLocale(option.code)"
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
