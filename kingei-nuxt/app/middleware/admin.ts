export default defineNuxtRouteMiddleware(() => {
  const { isAdmin } = useInstance()
  if (isAdmin.value) {
    return
  }
  return navigateTo('/settings/account')
})
