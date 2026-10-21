<script setup lang="ts">
import type { Friend } from '~/types/api'
import { formatDate } from '~/utils/display'

const props = defineProps<{
  friend: Friend
}>()

const { t, locale } = useI18n()
const { isMobile } = useIsMobile()
const chats = useChats()
const friends = useFriends()
const toast = useToast()
const pending = ref(false)

async function message() {
  pending.value = true
  try {
    const chat = await chats.openDirect(props.friend.friend_id)
    await navigateTo(`/chat/${chat.id}`)
  }
  catch (err) {
    toast.error(err)
    pending.value = false
  }
}

async function remove() {
  if (!confirm(t('friends.removeConfirm'))) {
    return
  }
  pending.value = true
  try {
    await friends.remove(props.friend.friend_id)
    toast.success(t('toast.removed'))
    await navigateTo('/friends')
  }
  catch (err) {
    toast.error(err)
    pending.value = false
  }
}
</script>

<template>
  <section class="profile">
    <header
      v-if="isMobile"
      class="top"
    >
      <NuxtLink
        to="/friends"
        class="icon-btn"
        :aria-label="$t('app.back')"
      >
        <AppIcon name="back" />
      </NuxtLink>
      <strong>{{ $t('friends.title') }}</strong>
    </header>
    <div class="stack">
      <div class="hero">
        <Avatar
          :name="friend.username"
          :seed="friend.friend_id"
          :size="88"
        />
        <h1>{{ friend.username }}</h1>
        <div class="actions">
          <button
            class="btn btn-primary"
            type="button"
            :disabled="pending"
            @click="message"
          >
            {{ $t('friends.message') }}
          </button>
          <button
            class="btn btn-secondary"
            type="button"
            :disabled="pending"
            @click="remove"
          >
            {{ $t('friends.remove') }}
          </button>
        </div>
      </div>
      <dl>
        <div>
          <dt>{{ $t('friends.email') }}</dt>
          <dd>{{ friend.email }}</dd>
        </div>
        <div>
          <dt>{{ $t('friends.since') }}</dt>
          <dd>{{ formatDate(friend.created_at, locale) }}</dd>
        </div>
      </dl>
    </div>
  </section>
</template>

<style scoped>
.profile {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: auto;
}

.stack {
  margin: auto;
  width: 100%;
}

.top {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 52px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-border);
}

.hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  text-align: center;
}

.hero h1 {
  font-size: 22px;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

dl {
  width: min(520px, 100%);
  margin: 24px auto;
  padding: 0 24px;
}

dl div {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
}

dt {
  color: var(--color-muted);
}

dd {
  margin: 0;
}
</style>
