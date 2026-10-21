<script setup lang="ts">
import type { Friend, FriendRequest } from '~/types/api'
import { shortId } from '~/utils/display'

const route = useRoute()
const { t } = useI18n()
const { open } = useAppActions()
const friends = useFriends()
const users = useUserDirectory()

const toast = useToast()
const query = ref('')

async function run(action: () => Promise<void>, ok: string) {
  try {
    await action()
    toast.success(t(ok))
  }
  catch (err) {
    toast.error(err)
  }
}

const filteredFriends = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) {
    return friends.friends.value
  }
  return friends.friends.value.filter(item =>
    item.username.toLowerCase().includes(q) || item.email.toLowerCase().includes(q),
  )
})

function requestName(request: FriendRequest, side: 'sender' | 'receiver') {
  const id = side === 'sender' ? request.sender_id : request.receiver_id
  return users.label(id, `${t('app.unknownUser')} ${shortId(id)}`)
}

function requestSeed(request: FriendRequest, side: 'sender' | 'receiver') {
  return side === 'sender' ? request.sender_id : request.receiver_id
}

function isFriendActive(id: string) {
  return String(route.params.id) === id
}
</script>

<template>
  <section class="list-pane">
    <header>
      <h1>{{ $t('friends.title') }}</h1>
      <button
        type="button"
        class="icon-btn"
        :title="$t('friends.add')"
        @click="open('friend')"
      >
        <AppIcon name="plus" />
      </button>
    </header>
    <div class="search">
      <input
        v-model="query"
        class="input"
        :placeholder="$t('friends.searchPlaceholder')"
      >
    </div>
    <div class="scroll">
      <template v-if="friends.incoming.value.length">
        <p class="list-section-label">
          {{ $t('friends.incoming') }} {{ friends.incoming.value.length }}
        </p>
        <div
          v-for="request in friends.incoming.value"
          :key="`in-${request.id}`"
          class="item"
        >
          <Avatar
            :name="requestName(request, 'sender')"
            :seed="requestSeed(request, 'sender')"
            :size="40"
          />
          <div class="meta">
            <strong
              class="truncate"
              :title="requestName(request, 'sender')"
            >{{ requestName(request, 'sender') }}</strong>
            <div class="row-actions">
              <button
                type="button"
                class="link"
                @click="run(() => friends.accept(request.id), 'toast.accepted')"
              >
                {{ $t('friends.accept') }}
              </button>
              <button
                type="button"
                class="link danger"
                @click="run(() => friends.reject(request.id), 'toast.rejected')"
              >
                {{ $t('friends.reject') }}
              </button>
            </div>
          </div>
        </div>
      </template>
      <template v-if="friends.outgoing.value.length">
        <p class="list-section-label">
          {{ $t('friends.outgoing') }} {{ friends.outgoing.value.length }}
        </p>
        <div
          v-for="request in friends.outgoing.value"
          :key="`out-${request.id}`"
          class="item"
        >
          <Avatar
            :name="requestName(request, 'receiver')"
            :seed="requestSeed(request, 'receiver')"
            :size="40"
          />
          <div class="meta">
            <strong
              class="truncate"
              :title="requestName(request, 'receiver')"
            >{{ requestName(request, 'receiver') }}</strong>
            <button
              type="button"
              class="link"
              @click="run(() => friends.cancel(request.id), 'toast.cancelled')"
            >
              {{ $t('friends.cancelRequest') }}
            </button>
          </div>
        </div>
      </template>
      <p class="list-section-label">
        {{ $t('friends.list') }} {{ filteredFriends.length }}
      </p>
      <p
        v-if="!filteredFriends.length && !friends.loading.value"
        class="muted pad"
      >
        {{ $t('friends.empty') }}
      </p>
      <NuxtLink
        v-for="friend in filteredFriends"
        :key="friend.friend_id"
        :to="`/friends/${friend.friend_id}`"
        class="item"
        :class="{ active: isFriendActive(friend.friend_id) }"
      >
        <Avatar
          :name="friend.username"
          :seed="friend.friend_id"
          :size="40"
        />
        <div class="meta">
          <strong
            class="truncate"
            :title="friend.username"
          >{{ friend.username }}</strong>
          <span
            class="muted truncate"
            :title="friend.email"
          >{{ friend.email }}</span>
        </div>
      </NuxtLink>
    </div>
  </section>
</template>

<style scoped>
.list-pane {
  display: flex;
  flex-direction: column;
  width: var(--session-width);
  min-width: var(--session-width);
  border-right: 1px solid var(--color-border);
  min-height: 0;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 16px 8px;
}

h1 {
  font-size: 18px;
}

.search {
  padding: 0 12px 8px;
}

.scroll {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  overflow: auto;
  padding: 4px 8px 16px;
}

.item {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px;
  border-radius: var(--radius-lg);
  color: inherit;
}

.item.active,
.item:hover {
  background: var(--bg-active);
}

.meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.row-actions {
  display: flex;
  gap: 10px;
}

.link {
  color: var(--color-primary);
  font-size: 13px;
}

.link.danger {
  color: var(--color-danger);
}

.pad {
  padding: 8px 16px;
}

@media (max-width: 899px) {
  .list-pane {
    width: 100%;
    min-width: 0;
    border-right: 0;
  }
}
</style>
