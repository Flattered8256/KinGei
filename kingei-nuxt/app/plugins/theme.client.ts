export default defineNuxtPlugin(() => {
  const { init } = useTheme()
  const mobile = useIsMobile()
  init()
  mobile.init()
})
