import { navigateTo, useNuxtApp, useRequestEvent, useRequestURL } from "#imports";
import { localePath, switchLocalePath } from "./routing/routing.js";
import { createNavigationResolver } from "./routing/navigation.js";
import { useNuxtI18nContext } from "./context.js";
import { useComposableContext } from "./composable-context.js";
import { isSupportedLocale } from "./shared/locales.js";
import { createLocaleDetector, useDetectors } from "./shared/detection.js";
import { useI18nDetection } from "./shared/utils.js";
import { resolveLocaleReach } from "./shared/domain.js";
export async function loadAndSetLocale(nuxtApp, locale) {
  const ctx = useNuxtI18nContext(nuxtApp);
  const oldLocale = ctx.getLocale();
  if (locale === oldLocale && !ctx.initial && (!ctx.vueI18n.__pendingLocale || ctx.vueI18n.__pendingLocale === locale)) {
    return locale;
  }
  const data = { oldLocale, newLocale: locale, initialSetup: ctx.initial, context: nuxtApp };
  let override = await nuxtApp.callHook("i18n:beforeLocaleSwitch", data);
  if (override != null && import.meta.dev) {
    console.warn("[nuxt-i18n] Do not return in `i18n:beforeLocaleSwitch`, mutate `data.newLocale` instead.");
  }
  override ??= data.newLocale;
  if (isSupportedLocale(override)) {
    locale = override;
  }
  await ctx.loadMessages(locale);
  await ctx.setLocaleSuspend(locale);
  return locale;
}
export function detectLocale(nuxtApp, route) {
  const detectConfig = useI18nDetection(nuxtApp);
  const detectors = useDetectors(useRequestEvent(nuxtApp), detectConfig, nuxtApp);
  const ctx = useNuxtI18nContext(nuxtApp);
  const detect = createLocaleDetector({
    detection: detectConfig,
    routing: __I18N_ROUTING__,
    domains: __I18N_DOMAINS__
  });
  return detect(detectors, route, ctx.initial) || ctx.getLocale() || ctx.getDefaultLocale() || "";
}
export function navigate(nuxtApp, to, locale) {
  if (!__I18N_ROUTING__) {
    return;
  }
  const ctx = useNuxtI18nContext(nuxtApp);
  const _ctx = useComposableContext(nuxtApp);
  const detectors = useDetectors(useRequestEvent(), useI18nDetection(nuxtApp), nuxtApp);
  const host = __I18N_DOMAINS__ ? useRequestURL({ xForwardedHost: true }).host : "";
  const resolve = createNavigationResolver({
    localeReach: __I18N_DOMAINS__ ? (locale2) => resolveLocaleReach(_ctx.getLocales(), host, locale2) : void 0,
    rootRedirect: ctx.rootRedirect,
    redirectStatusCode: ctx.redirectStatusCode,
    localePath: (path, locale2) => localePath(_ctx, path, locale2),
    switchLocalePath: (locale2, route) => switchLocalePath(_ctx, locale2, route),
    routeLocale: (route) => detectors.route(route),
    hasRoute: (name) => _ctx.router.hasRoute(name),
    getLocaleCodes: () => _ctx.getLocales().map((locale2) => locale2.code),
    strategy: __I18N_STRATEGY__,
    compactRoutes: __I18N_COMPACT_ROUTES__
  });
  const resolved = resolve(to, locale, !!ctx.vueI18n.__pendingLocale && !!useNuxtApp()._processingMiddleware);
  if (resolved) {
    return navigateTo(resolved.path, { redirectCode: resolved.code, external: resolved.external });
  }
}
