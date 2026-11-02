<script setup lang="ts">
import type { User } from '~/types/api'

const { t } = useI18n()
const toast = useToast()
const { modal, close } = useAppActions()
const { canAddFriend, canCreateGroup } = useInstance()
const chats = useChats()
const friends = useFriends()
const { user, updatePassword } = useAuth()

const groupName = ref('')
const groupPending = ref(false)

const email = ref('')
const found = ref<User | null>(null)
const friendPending = ref(false)

const oldPassword = ref('')
const newPassword = ref('')
const passwordPending = ref(false)

watch(modal, () => {
  groupName.value = ''
  email.value = ''
  found.value = null
  oldPassword.value = ''
  newPassword.value = ''
})

async function createGroup() {
  if (!groupName.value.trim()) {
    toast.error(t('error.EmptyName'))
    return
  }
  groupPending.value = true
  try {
    const chat = await chats.openGroup(groupName.value.trim())
    toast.success(t('toast.groupCreated'))
    close()
    await navigateTo(`/chat/${chat.id}`)
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    groupPending.value = false
  }
}

async function findFriend() {
  found.value = null
  if (!email.value.trim()) {
    toast.error(t('auth.required'))
    return
  }
  friendPending.value = true
  try {
    found.value = await friends.lookup(email.value.trim())
    if (found.value.id === user.value?.id) {
      found.value = null
      toast.error(t('error.CannotAddSelf'))
    }
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    friendPending.value = false
  }
}

async function sendRequest() {
  if (!found.value) {
    return
  }
  friendPending.value = true
  try {
    await friends.request(found.value.id)
    toast.success(t('toast.requestSent'))
    close()
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    friendPending.value = false
  }
}

async function changePassword() {
  if (!oldPassword.value || !newPassword.value) {
    toast.error(t('auth.required'))
    return
  }
  passwordPending.value = true
  try {
    await updatePassword({
      old_password: oldPassword.value,
      new_password: newPassword.value,
    })
    toast.success(t('toast.passwordChanged'))
    close()
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    passwordPending.value = false
  }
}

</script>

<template>
  <KgModal
    :open="modal === 'group' && canCreateGroup"
    :title="$t('chat.createGroup')"
    @close="close"
  >
    <form
      class="stack"
      @submit.prevent="createGroup"
    >
      <label class="field">
        <span class="field-label">{{ $t('chat.groupName') }}</span>
        <input
          v-model="groupName"
          class="input"
          :placeholder="$t('chat.groupNamePlaceholder')"
        >
      </label>
      <div class="modal-actions">
        <button
          class="btn btn-secondary"
          type="button"
          @click="close"
        >
          {{ $t('app.cancel') }}
        </button>
        <button
          class="btn btn-primary"
          type="submit"
          :disabled="groupPending"
        >
          {{ $t('app.confirm') }}
        </button>
      </div>
    </form>
  </KgModal>

  <KgModal
    :open="modal === 'friend' && canAddFriend"
    :title="$t('friends.add')"
    @close="close"
  >
    <form
      class="stack"
      @submit.prevent="found ? sendRequest() : findFriend()"
    >
      <label class="field">
        <span class="field-label">{{ $t('auth.email') }}</span>
        <input
          v-model="email"
          class="input"
          type="email"
          :placeholder="$t('friends.emailPlaceholder')"
        >
      </label>
      <div
        v-if="found"
        class="found"
      >
        <Avatar
          :name="found.username"
          :seed="found.id"
          :file-id="found.avatar"
          :size="40"
        />
        <div>
          <strong>{{ found.username }}</strong>
          <p class="muted">
            {{ found.email }}
          </p>
        </div>
      </div>
      <div class="modal-actions">
        <button
          class="btn btn-secondary"
          type="button"
          @click="close"
        >
          {{ $t('app.cancel') }}
        </button>
        <button
          class="btn btn-primary"
          type="submit"
          :disabled="friendPending"
        >
          {{ found ? $t('friends.sendRequest') : $t('friends.find') }}
        </button>
      </div>
    </form>
  </KgModal>

  <KgModal
    :open="modal === 'password'"
    :title="$t('settings.changePassword')"
    @close="close"
  >
    <form
      class="stack"
      @submit.prevent="changePassword"
    >
      <label class="field">
        <span class="field-label">{{ $t('settings.oldPassword') }}</span>
        <input
          v-model="oldPassword"
          class="input"
          type="password"
          autocomplete="current-password"
        >
      </label>
      <label class="field">
        <span class="field-label">{{ $t('settings.newPassword') }}</span>
        <input
          v-model="newPassword"
          class="input"
          type="password"
          autocomplete="new-password"
        >
      </label>
      <div class="modal-actions">
        <button
          class="btn btn-secondary"
          type="button"
          @click="close"
        >
          {{ $t('app.cancel') }}
        </button>
        <button
          class="btn btn-primary"
          type="submit"
          :disabled="passwordPending"
        >
          {{ $t('app.save') }}
        </button>
      </div>
    </form>
  </KgModal>
</template>

<style scoped>
.stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.found {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  border-radius: var(--radius-md);
  background: var(--bg-muted);
}
</style>
