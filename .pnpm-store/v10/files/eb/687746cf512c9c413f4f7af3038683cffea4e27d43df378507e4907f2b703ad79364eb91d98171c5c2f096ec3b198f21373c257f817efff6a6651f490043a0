import { useHead, useNuxtApp, useRequestURL, useRouter } from "#imports";
import { createRoutingContext } from "./routing/context.js";
export function useComposableContext(nuxtApp) {
  const context = nuxtApp?._nuxtI18n?.composableCtx;
  if (!context) {
    throw new Error(
      "i18n context is not initialized. Ensure the i18n plugin is installed and the composable is used within a Vue component or setup function."
    );
  }
  return context;
}
export function createComposableContext(ctx, nuxtApp = useNuxtApp()) {
  const localePathPayload = getLocalePathPayload(nuxtApp);
  const routingCtx = createRoutingContext({
    router: useRouter(),
    defaultLocale: ctx.getDefaultLocale(),
    configuredDefaultLocale: ctx.config.defaultLocale || "",
    getLocale: ctx.getLocale,
    getLocales: ctx.getLocales,
    getBaseUrl: ctx.getBaseUrl,
    getCanonicalBaseUrl: ctx.getCanonicalBaseUrl,
    getHost: () => useRequestURL({ xForwardedHost: true }).host,
    getLocalePathPayload: () => __I18N_STRICT_SEO__ && import.meta.client && nuxtApp.isHydrating && localePathPayload,
    strategy: __I18N_STRATEGY__,
    routing: __I18N_ROUTING__,
    domains: __I18N_DOMAINS__,
    trailingSlash: __TRAILING_SLASH__,
    strictSeo: __I18N_STRICT_SEO__,
    compactRoutes: __I18N_COMPACT_ROUTES__
  });
  return {
    ...routingCtx,
    strictSeo: __I18N_STRICT_SEO__,
    _head: void 0,
    get head() {
      this._head ??= useHead({});
      return this._head;
    },
    metaState: { htmlAttrs: {}, meta: [], link: [] },
    seoSettings: {
      dir: __I18N_STRICT_SEO__,
      lang: __I18N_STRICT_SEO__,
      // the object form carries `canonicalQueries`, `__I18N_STRICT_SEO__` only compiles its truthiness
      seo: typeof ctx.config.experimental.strictSeo === "object" && ctx.config.experimental.strictSeo || __I18N_STRICT_SEO__
    },
    localePathPayload,
    routingOptions: {
      // `x-default` annotates the page cluster, so every domain has to agree on it - unlike the
      // routing `defaultLocale` above, which is the current host's unprefixed locale
      defaultLocale: ctx.config.defaultLocale || "",
      domains: __I18N_DOMAINS__,
      strictCanonicals: ctx.config.experimental.alternateLinkCanonicalQueries ?? true,
      hreflangLinks: !(!__I18N_ROUTING__ && !__I18N_DOMAINS__)
    }
  };
}
function getLocalePathPayload(nuxtApp = useNuxtApp()) {
  const payload = import.meta.client && document.querySelector(`[data-nuxt-i18n-slp="${nuxtApp._id}"]`)?.textContent;
  return JSON.parse(payload || "{}");
}
