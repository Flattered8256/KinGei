<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const { t } = useI18n()
const { user, updateProfile } = useAuth()
const { isMobile } = useIsMobile()
const { open } = useAppActions()
const apiI18n = useApiI18n()

const username = ref(user.value?.username ?? '')
const email = ref(user.value?.email ?? '')
const pending = ref(false)
const error = ref('')
const saved = ref(false)

watch(user, (next) => {
  username.value = next?.username ?? ''
  email.value = next?.email ?? ''
})

async function save() {
  error.value = ''
  saved.value = false
  if (!username.value.trim() || !email.value.trim()) {
    error.value = t('auth.required')
    return
  }
  pending.value = true
  try {
    await updateProfile({
      username: username.value.trim(),
      email: email.value.trim(),
    })
    saved.value = true
  }
  catch (err) {
    error.value = apiI18n.fromError(err)
  }
  finally {
    pending.value = false
  }
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
      <h1>{{ $t('settings.account') }}</h1>
    </header>
    <div
      v-if="user"
      class="hero"
    >
      <Avatar
        :name="user.username"
        :seed="user.id"
        :size="56"
      />
      <div>
        <strong>{{ user.username }}</strong>
        <p class="muted">
          {{ user.email }}
        </p>
      </div>
    </div>
    <form
      class="form"
      @submit.prevent="save"
    >
      <label class="field">
        <span class="field-label">{{ $t('settings.username') }}</span>
        <input
          v-model="username"
          class="input"
          autocomplete="username"
        >
      </label>
      <label class="field">
        <span class="field-label">{{ $t('settings.email') }}</span>
        <input
          v-model="email"
          class="input"
          type="email"
          autocomplete="email"
        >
      </label>
      <p
        v-if="error"
        class="form-error"
      >
        {{ error }}
      </p>
      <p
        v-if="saved"
        class="muted"
      >
        {{ $t('settings.saved') }}
      </p>
      <div class="actions">
        <button
          class="btn btn-primary"
          type="submit"
          :disabled="pending"
        >
          {{ $t('app.save') }}
        </button>
        <button
          class="btn btn-secondary"
          type="button"
          @click="open('password')"
        >
          {{ $t('settings.changePassword') }}
        </button>
      </div>
    </form>
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

.hero {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 12px 0 20px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 520px;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }

  .actions .btn {
    flex: 1;
  }
}
</style>
