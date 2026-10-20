export default defineEventHandler((event) => {
  const config = useRuntimeConfig()
  const requestUrl = getRequestURL(event)
  const target = String(config.apiProxyTarget).replace(/\/$/, '')
  const path = requestUrl.pathname.replace(/^\/api/, '') || '/'
  return proxyRequest(event, `${target}${path}${requestUrl.search}`)
})
