<script setup lang="ts">
definePageMeta({
  layout: 'auth',
  middleware: 'guest',
})

const { t, locale, setLocale } = useI18n()
const { login } = useAuth()
const toast = useToast()

const email = ref('')
const password = ref('')
const pending = ref(false)

async function submit() {
  if (!email.value.trim() || !password.value) {
    toast.error(t('auth.required'))
    return
  }
  pending.value = true
  try {
    await login({ email: email.value.trim(), password: password.value })
    await navigateTo('/chat')
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pending.value = false
  }
}

function toggleLocale() {
  void setLocale(locale.value === 'zh-CN' ? 'en' : 'zh-CN')
}
</script>

<template>
  <div class="auth-wrap">
    <form
      class="auth-card"
      @submit.prevent="submit"
    >
      <div class="brand">
        <Avatar
          name="K"
          :size="56"
          :hash="false"
        />
        <h1>{{ $t('auth.loginTitle', { name: $t('app.name') }) }}</h1>
      </div>
      <label class="field">
        <span class="field-label">{{ $t('auth.email') }}</span>
        <input
          v-model="email"
          class="input"
          type="email"
          autocomplete="email"
          :placeholder="$t('auth.emailPlaceholder')"
        >
      </label>
      <label class="field">
        <span class="field-label">{{ $t('auth.password') }}</span>
        <input
          v-model="password"
          class="input"
          type="password"
          autocomplete="current-password"
          :placeholder="$t('auth.passwordPlaceholder')"
        >
      </label>
      <button
        class="btn btn-primary btn-block"
        type="submit"
        :disabled="pending"
      >
        {{ $t('auth.login') }}
      </button>
      <p class="auth-switch">
        {{ $t('auth.noAccount') }}
        <NuxtLink to="/register">
          {{ $t('auth.register') }}
        </NuxtLink>
      </p>
    </form>
    <button
      type="button"
      class="lang"
      @click="toggleLocale"
    >
      {{ locale === 'zh-CN' ? $t('settings.zh') : $t('settings.en') }}
    </button>
  </div>
</template>

<style scoped>
.auth-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  width: min(420px, 100%);
}

.auth-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  padding: 32px 28px 24px;
  background: var(--bg-panel);
  border-radius: 20px;
  box-shadow: var(--shadow-card);
}

.brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.brand h1 {
  font-size: 22px;
  font-weight: 700;
  text-align: center;
}

.auth-switch {
  text-align: center;
  color: var(--color-muted);
  font-size: 13px;
}

.lang {
  color: var(--color-muted);
  font-size: 13px;
}
</style>
