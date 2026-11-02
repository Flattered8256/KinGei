<script setup lang="ts">
import { getInvite, regenerateInvite } from '~/utils/api/admin'
import { isApiErrorCode } from '~/utils/api/error'

definePageMeta({
  layout: 'app',
  middleware: ['auth', 'admin'],
})

const { t } = useI18n()
const { isMobile } = useIsMobile()
const { instance, updateFlags } = useInstance()
const toast = useToast()

type FlagKey = 'allow_add_friend' | 'allow_create_group' | 'invite_only'

let queue: Promise<void> = Promise.resolve()

const code = ref<string | null>(null)
const codeLoaded = ref(false)
const pendingCode = ref(false)

onMounted(() => {
  void loadCode()
})

async function loadCode() {
  try {
    const invite = await getInvite()
    code.value = invite.code
  }
  catch (err) {
    if (isApiErrorCode(err, 'InviteNotFound')) {
      code.value = null
    }
    else {
      toast.error(err)
    }
  }
  finally {
    codeLoaded.value = true
  }
}

async function generate() {
  pendingCode.value = true
  try {
    const invite = await regenerateInvite()
    code.value = invite.code
    toast.success(t('settings.saved'))
  }
  catch (err) {
    toast.error(err)
  }
  finally {
    pendingCode.value = false
  }
}

async function copyCode() {
  if (!code.value) {
    return
  }
  try {
    await navigator.clipboard.writeText(code.value)
    toast.success(t('settings.inviteCodeCopied'))
  }
  catch (err) {
    toast.error(err)
  }
}

function setFlag(key: FlagKey, value: boolean) {
  if (instance.value[key] === value) {
    return
  }
  instance.value = { ...instance.value, [key]: value }
  queue = queue.then(() => persist(key, value))
}

async function persist(key: FlagKey, sent: boolean) {
  if (instance.value[key] !== sent) {
    return
  }
  try {
    await updateFlags({ [key]: sent })
    toast.success(t('settings.saved'))
  }
  catch (err) {
    if (instance.value[key] === sent) {
      instance.value = { ...instance.value, [key]: !sent }
    }
    toast.error(err)
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
      <h1>{{ $t('settings.site') }}</h1>
    </header>
    <div class="box">
      <div class="row">
        <div class="copy">
          <strong>{{ $t('settings.allowAddFriend') }}</strong>
          <span class="muted">{{ $t('settings.allowAddFriendHint') }}</span>
        </div>
        <KgSwitch
          :model-value="instance.allow_add_friend"
          :aria-label="$t('settings.allowAddFriend')"
          @update:model-value="setFlag('allow_add_friend', $event)"
        />
      </div>
      <div class="divider" />
      <div class="row">
        <div class="copy">
          <strong>{{ $t('settings.allowCreateGroup') }}</strong>
          <span class="muted">{{ $t('settings.allowCreateGroupHint') }}</span>
        </div>
        <KgSwitch
          :model-value="instance.allow_create_group"
          :aria-label="$t('settings.allowCreateGroup')"
          @update:model-value="setFlag('allow_create_group', $event)"
        />
      </div>
      <div class="divider" />
      <div class="row">
        <div class="copy">
          <strong>{{ $t('settings.inviteOnly') }}</strong>
          <span class="muted">{{ $t('settings.inviteOnlyHint') }}</span>
        </div>
        <KgSwitch
          :model-value="instance.invite_only"
          :aria-label="$t('settings.inviteOnly')"
          @update:model-value="setFlag('invite_only', $event)"
        />
      </div>
    </div>
    <div class="box invite-box">
      <strong>{{ $t('settings.inviteCode') }}</strong>
      <div
        v-if="codeLoaded"
        class="invite-body"
      >
        <template v-if="code">
          <p class="invite-code">
            {{ code }}
          </p>
          <div class="invite-actions">
            <button
              class="btn btn-secondary"
              type="button"
              :disabled="pendingCode"
              @click="copyCode"
            >
              {{ $t('settings.inviteCodeCopy') }}
            </button>
            <button
              class="btn btn-secondary"
              type="button"
              :disabled="pendingCode"
              @click="generate"
            >
              {{ $t('settings.inviteCodeRegenerate') }}
            </button>
          </div>
        </template>
        <template v-else>
          <p class="muted">
            {{ $t('settings.inviteCodeEmpty') }}
          </p>
          <div class="invite-actions">
            <button
              class="btn btn-secondary"
              type="button"
              :disabled="pendingCode"
              @click="generate"
            >
              {{ $t('settings.inviteCodeGenerate') }}
            </button>
          </div>
        </template>
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

h1 {
  font-size: 22px;
}

.box {
  max-width: 560px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
}

.copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.copy strong {
  font-size: 14px;
}

.divider {
  height: 1px;
  background: var(--color-border);
}

.invite-box {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 24px;
  padding-top: 8px;
}

.invite-box strong {
  font-size: 14px;
}

.invite-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.invite-code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.16em;
}

.invite-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }
}
</style>
