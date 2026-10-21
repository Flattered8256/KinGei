<script setup lang="ts">
definePageMeta({
  layout: 'app',
  middleware: 'auth',
})

const route = useRoute()
const friends = useFriends()

const friend = computed(() => friends.friendById(String(route.params.id)))
</script>

<template>
  <FriendProfile
    v-if="friend"
    :friend="friend"
  />
  <div
    v-else
    class="empty-state"
  >
    <p>{{ friends.loading.value ? $t('app.loading') : $t('friends.notFound') }}</p>
  </div>
</template>
