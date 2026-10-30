<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: ['auth', 'admin'],
})

const { t } = useI18n()
const { isMobile } = useIsMobile()
const { instance, updateFlags } = useInstance()
const toast = useToast()

type FlagKey = 'allow_add_friend' | 'allow_create_group'

let queue: Promise<void> = Promise.resolve()

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
    <p class="hint">
      {{ $t('settings.siteHint') }}
    </p>
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

.hint {
  max-width: 560px;
  margin-bottom: 16px;
  color: var(--color-muted);
  font-size: 13px;
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

@media (max-width: 899px) {
  .page {
    padding: 0 16px 24px;
  }
}
</style>
