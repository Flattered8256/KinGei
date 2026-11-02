export default defineNuxtRouteMiddleware(() => {
  const { isAuthenticated } = useAuth()
  if (isAuthenticated.value) {
    return
  }
  const { instance } = useInstance()
  return navigateTo(instance.value.initializing ? '/register' : '/login')
})
