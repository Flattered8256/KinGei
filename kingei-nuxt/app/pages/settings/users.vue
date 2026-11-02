<script setup lang="ts">
import type { User } from '~/types/api'
import { deleteAdminUser, listAdminUsers } from '~/utils/api/admin'

definePageMeta({
  layout: 'app',
  middleware: ['auth', 'admin'],
})

const { t } = useI18n()
const { isMobile } = useIsMobile()
const { user: me } = useAuth()
const toast = useToast()

const users = ref<User[]>([])
const loading = ref(false)
const removingId = ref<string | null>(null)

onMounted(() => {
  void load()
})

async function load() {
  loading.value = true
  try {
    users.value = await listAdminUsers()
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    loading.value = false
  }
}

function canRemove(user: User) {
  return user.role !== 'Admin' && user.id !== me.value?.id
}

async function remove(user: User) {
  if (!canRemove(user)) {
    return
  }
  if (!confirm(t('settings.removeUserConfirm'))) {
    return
  }
  removingId.value = user.id
  try {
    await deleteAdminUser(user.id)
    users.value = users.value.filter(item => item.id !== user.id)
    toast.success(t('settings.userRemoved'))
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    removingId.value = null
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
      <div class="heading">
        <h1>{{ $t('settings.users') }}</h1>
        <p
          v-if="!isMobile"
          class="muted"
        >
          {{ $t('settings.usersHint') }}
        </p>
      </div>
    </header>
    <div class="list">
      <KgOverlay v-if="loading && !users.length" />
      <p
        v-else-if="!users.length"
        class="empty muted"
      >
        {{ $t('app.empty') }}
      </p>
      <div
        v-for="user in users"
        :key="user.id"
        class="row"
      >
        <Avatar
          :name="user.username"
          :seed="user.id"
          :file-id="user.avatar"
          :size="36"
        />
        <div class="meta">
          <div class="name-row">
            <strong class="truncate">{{ user.username }}</strong>
            <span
              class="badge"
              :class="user.role === 'Admin' ? 'admin' : 'member'"
            >
              {{ $t(`enum.userRole.${user.role}`) }}
            </span>
          </div>
          <span class="email truncate">{{ user.email }}</span>
        </div>
        <button
          v-if="canRemove(user)"
          type="button"
          class="remove"
          :disabled="removingId === user.id"
          @click="remove(user)"
        >
          {{ $t('settings.removeUser') }}
        </button>
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

.heading {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

h1 {
  font-size: 22px;
}

.heading .muted {
  font-size: 13px;
}

.list {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 560px;
  min-height: 120px;
  margin-top: 12px;
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.name-row strong {
  font-size: 15px;
}

.email {
  color: var(--color-muted);
  font-size: 13px;
}

.badge {
  flex-shrink: 0;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  line-height: 15px;
}

.badge.admin {
  background: var(--badge-admin-bg);
  color: var(--badge-admin-fg);
}

.badge.member {
  background: var(--badge-member-bg);
  color: var(--badge-member-fg);
}

.remove {
  flex-shrink: 0;
  color: var(--color-muted);
  font-size: 13px;
  font-weight: 500;
}

.remove:hover:not(:disabled) {
  color: var(--color-danger);
}

.empty {
  padding: 24px 0;
}

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }
}
</style>
