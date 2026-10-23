<script setup lang="ts">
import type { Chat } from '~/types/api'

const props = defineProps<{
  chat: Chat
}>()

const { t } = useI18n()
const { user } = useAuth()
const chats = useChats()
const friends = useFriends()
const toast = useToast()

const inviteOpen = ref(false)
const renameOpen = ref(false)
const newName = ref('')
const pending = ref(false)

const myId = computed(() => user.value?.id ?? '')
const isOwner = computed(() => props.chat.owner_id === myId.value)
const isGroup = computed(() => props.chat.type === 'Group')

const members = computed(() =>
  [...props.chat.member].sort((a, b) => {
    if (a.user.id === props.chat.owner_id) {
      return -1
    }
    if (b.user.id === props.chat.owner_id) {
      return 1
    }
    return a.user.username.localeCompare(b.user.username)
  }),
)

const inviteCandidates = computed(() => {
  const memberIds = new Set(props.chat.member.map(item => item.user.id))
  return friends.friends.value.filter(friend => !memberIds.has(friend.user.id))
})

watch(
  () => props.chat.id,
  () => {
    newName.value = props.chat.name ?? ''
    inviteOpen.value = false
    renameOpen.value = false
  },
  { immediate: true },
)

async function invite(userId: string) {
  if (!userId) {
    return
  }

  pending.value = true
  try {
    await chats.invite(props.chat.id, userId)
    toast.success(t('toast.invited'))
    inviteOpen.value = false
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pending.value = false
  }
}

async function rename() {
  pending.value = true
  try {
    await chats.rename(props.chat.id, newName.value.trim())
    toast.success(t('toast.renamed'))
    renameOpen.value = false
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pending.value = false
  }
}

async function kick(userId: string) {
  if (!confirm(t('chat.kickConfirm'))) {
    return
  }
  pending.value = true
  try {
    await chats.removeMember(props.chat.id, userId)
    toast.success(t('toast.kicked'))
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pending.value = false
  }
}

async function leave() {
  if (!confirm(t('chat.leaveConfirm'))) {
    return
  }
  pending.value = true
  try {
    await chats.removeMember(props.chat.id, myId.value)
    toast.success(t('toast.left'))
    await navigateTo('/chat')
  }
  catch (err) {
    toast.error(err)
    pending.value = false
  }
}
</script>

<template>
  <aside
    v-if="isGroup"
    class="panel"
  >
    <header>
      <strong>{{ $t('chat.members') }} {{ members.length }}</strong>
      <button
        v-if="inviteCandidates.length"
        type="button"
        class="icon-btn"
        :title="$t('chat.invite')"
        @click="inviteOpen = true"
      >
        <AppIcon name="plus" :size="18" />
      </button>
    </header>
    <InviteFriendModal
      :open="inviteOpen"
      :friends="inviteCandidates"
      :pending="pending"
      @close="inviteOpen = false"
      @invite="invite"
    />
    <div
      v-if="renameOpen"
      class="box"
    >
      <input
        v-model="newName"
        class="input"
        :placeholder="$t('chat.groupNamePlaceholder')"
      >
      <button
        class="btn btn-primary btn-block"
        type="button"
        :disabled="pending"
        @click="rename"
      >
        {{ $t('app.save') }}
      </button>
    </div>
    <ul>
      <li
        v-for="member in members"
        :key="member.user.id"
      >
        <Avatar
          :name="member.user.username"
          :seed="member.user.id"
          :file-id="member.user.avatar"
          :size="36"
        />
        <div class="who">
          <strong
            class="truncate"
            :title="member.user.username"
          >{{ member.user.username }}</strong>
          <span
            v-if="member.user.id === chat.owner_id"
            class="role"
          >{{ $t('chat.owner') }}</span>
        </div>
        <button
          v-if="isOwner && member.user.id !== myId"
          type="button"
          class="tiny"
          @click="kick(member.user.id)"
        >
          {{ $t('chat.kick') }}
        </button>
      </li>
    </ul>
    <div class="footer">
      <button
        v-if="isOwner"
        type="button"
        class="btn btn-secondary btn-block"
        @click="renameOpen = !renameOpen"
      >
        {{ $t('chat.rename') }}
      </button>
      <button
        type="button"
        class="btn btn-danger btn-block"
        :disabled="pending"
        @click="leave"
      >
        {{ $t('chat.leave') }}
      </button>
    </div>
  </aside>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: var(--member-width);
  min-width: var(--member-width);
  min-height: 0;
  height: 100%;
  border-left: 1px solid var(--color-border);
  padding: 12px;
  gap: 10px;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

ul {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow: auto;
  flex: 1;
}

li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
}

.who {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.role {
  color: var(--color-primary);
  font-size: 12px;
}

.tiny {
  flex-shrink: 0;
  color: var(--color-danger);
  font-size: 12px;
}

.box {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: auto;
}
</style>
