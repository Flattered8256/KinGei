import { computed, getCurrentScope, onScopeDispose, ref, useHead, useRequestEvent, watch } from "#imports";
import { assign } from "@intlify/shared";
import { localeHead as _localeHead } from "#i18n-kit/head";
import { localeRoute, switchLocalePath } from "./routing.js";
function patchHead(head, input) {
  head?.patch(input);
}
export function missesClusterFallback(ctx, config) {
  const { domains, hreflangLinks, defaultLocale } = ctx.routingOptions;
  return !!config.seo && domains && hreflangLinks && !defaultLocale;
}
let warnedClusterFallback = false;
function createHeadContext(ctx, config, locale = ctx.getLocale(), locales = ctx.getLocales(), baseUrl = ctx.routingOptions.domains ? ctx.getBaseUrl(locale) : ctx.getBaseUrl()) {
  const currentLocale = locales.find((l) => l.code === locale) || { code: locale };
  const canonicalQueries = [...new Set(typeof config.seo === "object" && config.seo?.canonicalQueries || [])];
  if (import.meta.dev && !warnedClusterFallback && missesClusterFallback(ctx, config)) {
    warnedClusterFallback = true;
    console.warn("[nuxt-i18n] Set `defaultLocale` to annotate an `x-default` alternate for your domains.");
  }
  const alternateCtx = { ...ctx, afterSwitchLocalePath: ctx.getAlternatePath };
  if (!baseUrl && !ctx.routingOptions.domains) {
    if (ctx.strictSeo) {
      throw new Error("I18n `baseUrl` is required to generate valid SEO tag links.");
    }
    console.warn("I18n `baseUrl` is required to generate valid SEO tag links.");
  }
  return {
    ...config,
    key: ctx.strictSeo ? "key" : "id",
    strictSeo: ctx.strictSeo,
    locales,
    currentLocale: locale,
    baseUrl,
    canonicalQueries,
    hreflangLinks: ctx.routingOptions.hreflangLinks,
    defaultLocale: ctx.routingOptions.defaultLocale,
    strictCanonicals: ctx.strictSeo || ctx.routingOptions.strictCanonicals,
    getRouteBaseName: ctx.getRouteBaseName,
    getCurrentRoute: () => ctx.router.currentRoute.value,
    getCurrentLanguage: () => currentLocale.language,
    getCurrentDirection: () => currentLocale.dir || __DEFAULT_DIRECTION__,
    getLocaleRoute: (route) => localeRoute(ctx, route),
    getLocalizedRoute: (locale2, route) => switchLocalePath(alternateCtx, locale2, route),
    getRouteWithoutQuery: () => {
      try {
        return assign({}, ctx.router.resolve({ query: {} }), { meta: ctx.router.currentRoute.value.meta });
      } catch {
        return void 0;
      }
    }
  };
}
export function localeHead(ctx, { dir = true, lang = true, seo = true }) {
  return _localeHead(createHeadContext(ctx, { dir, lang, seo }));
}
export function _useLocaleHead(ctx, options) {
  const metaObject = ref(_localeHead(createHeadContext(ctx, options)));
  if (import.meta.client) {
    const unsub = watch([() => ctx.router.currentRoute.value, () => ctx.getLocale()], () => {
      metaObject.value = _localeHead(createHeadContext(ctx, options));
      ctx.strictSeo && patchHead(ctx.head, metaObject.value);
    });
    if (getCurrentScope()) {
      onScopeDispose(unsub);
    }
  }
  ctx.strictSeo && patchHead(ctx.head, metaObject.value);
  return metaObject;
}
export function _useSetI18nParams(ctx, seo, router = ctx.router) {
  const head = ctx.strictSeo ? ctx.head : useHead({});
  const evt = ctx.strictSeo && import.meta.server && useRequestEvent();
  const _i18nParams = ref({});
  const i18nParams = computed({
    get() {
      return router.currentRoute.value.meta[__DYNAMIC_PARAMS_KEY__];
    },
    set(val) {
      _i18nParams.value = val;
      router.currentRoute.value.meta[__DYNAMIC_PARAMS_KEY__] = val;
      if (evt && evt?.context.nuxtI18n?.slp) {
        evt.context.nuxtI18n.slp = val;
      }
    }
  });
  const unsub = watch(
    () => router.currentRoute.value.fullPath,
    () => {
      router.currentRoute.value.meta[__DYNAMIC_PARAMS_KEY__] = _i18nParams.value;
      ctx.strictSeo && updateState();
    }
  );
  if (getCurrentScope()) {
    onScopeDispose(unsub);
  }
  function updateState() {
    ctx.metaState = _localeHead(createHeadContext(ctx, ctxOptions.value));
    patchHead(head, ctx.metaState);
  }
  const ctxOptions = ref({
    ...ctx.seoSettings,
    key: ctx.strictSeo ? "key" : "id",
    seo: seo ?? ctx.seoSettings.seo
  });
  return function(params) {
    i18nParams.value = { ...params };
    ctx.strictSeo && updateState();
    if (!ctx.strictSeo) {
      const val = _localeHead(createHeadContext(ctx, ctxOptions.value));
      patchHead(head, val);
    }
  };
}
