export default defineNuxtRouteMiddleware((to) => {
  const { isAuthenticated } = useAuth()
  if (isAuthenticated.value) {
    return navigateTo('/chat')
  }
  const { instance } = useInstance()
  if (instance.value.initializing && to.path === '/login') {
    return navigateTo('/register')
  }
})
