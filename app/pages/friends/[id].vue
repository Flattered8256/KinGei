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
    class="empty-state profile-missing"
  >
    <KgOverlay v-if="friends.loading.value" />
    <p v-else>{{ $t('friends.notFound') }}</p>
  </div>
</template>

<style scoped>
.profile-missing {
  position: relative;
  flex: 1;
  width: 100%;
}
</style>
