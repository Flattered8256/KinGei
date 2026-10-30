export default defineNuxtPlugin(async () => {
  await useInstance().refresh()
})
