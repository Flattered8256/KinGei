<script setup lang="ts">
definePageMeta({
  layout: 'auth',
  middleware: 'guest',
})

const { t, locale, setLocale } = useI18n()
const { register, login } = useAuth()
const { instance, refresh: refreshInstance } = useInstance()
const toast = useToast()
const route = useRoute()

const username = ref('')
const email = ref('')
const password = ref('')
const confirm = ref('')
const inviteCode = ref(readQueryCode())
const pending = ref(false)

const needInvite = computed(() => instance.value.invite_only && !instance.value.initializing)

function readQueryCode() {
  const raw = route.query.code
  const value = Array.isArray(raw) ? raw[0] : raw
  return typeof value === 'string' ? value.trim() : ''
}

async function submit() {
  if (!username.value.trim() || !email.value.trim() || !password.value || !confirm.value) {
    toast.error(t('auth.required'))
    return
  }
  if (needInvite.value && !inviteCode.value.trim()) {
    toast.error(t('auth.required'))
    return
  }
  if (password.value !== confirm.value) {
    toast.error(t('auth.passwordMismatch'))
    return
  }
  pending.value = true
  try {
    await register({
      username: username.value.trim(),
      email: email.value.trim(),
      password: password.value,
      ...(needInvite.value ? { invite_code: inviteCode.value.trim() } : {}),
    })
    await refreshInstance()
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
        <h1>{{ $t(instance.initializing ? 'auth.registerAdminTitle' : 'auth.registerTitle', { name: $t('app.name') }) }}</h1>
      </div>
      <label class="field">
        <span class="field-label">{{ $t('auth.username') }}</span>
        <input
          v-model="username"
          class="input"
          autocomplete="username"
          :placeholder="$t('auth.usernamePlaceholder')"
        >
      </label>
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
          autocomplete="new-password"
          :placeholder="$t('auth.passwordPlaceholder')"
        >
      </label>
      <label class="field">
        <span class="field-label">{{ $t('auth.confirmPassword') }}</span>
        <input
          v-model="confirm"
          class="input"
          type="password"
          autocomplete="new-password"
          :placeholder="$t('auth.confirmPlaceholder')"
        >
      </label>
      <label
        v-if="needInvite"
        class="field"
      >
        <span class="field-label">{{ $t('auth.inviteCode') }}</span>
        <input
          v-model="inviteCode"
          class="input invite-input"
          autocomplete="off"
          autocapitalize="characters"
          spellcheck="false"
          maxlength="8"
          :placeholder="$t('auth.inviteCodePlaceholder')"
        >
      </label>
      <button
        class="btn btn-primary btn-block"
        type="submit"
        :disabled="pending"
      >
        {{ $t('auth.register') }}
      </button>
      <p
        v-if="!instance.initializing"
        class="auth-switch"
      >
        {{ $t('auth.hasAccount') }}
        <NuxtLink to="/login">
          {{ $t('auth.login') }}
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

.invite-input {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
</style>
