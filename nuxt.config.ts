export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  runtimeConfig: {
    apiProxyTarget: 'http://127.0.0.1:8000',
    public: {
      apiBase: '/api',
    },
  },
})
