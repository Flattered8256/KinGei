import { stringify } from "devalue";
import { defineI18nMiddleware } from "@intlify/h3";
import { defineNitroPlugin, useRuntimeConfig, useStorage } from "nitropack/runtime";
import { initializeI18nContext, tryUseI18nContext, useI18nContext } from "./context.js";
import { createUserLocaleDetector } from "./utils/locale-detector.js";
import { pickNested } from "./utils/messages-utils.js";
import { isSupportedLocale, resolveDefaultLocale } from "../shared/locales.js";
import { setupVueI18nOptions } from "../shared/vue-i18n.js";
import { joinURL, parsePath } from "ufo";
import { appId } from "#internal/nuxt.config.mjs";
import { localeDetector } from "#internal/i18n-locale-detector.mjs";
import { resolveRootRedirect, useI18nDetection, useRuntimeI18n } from "../shared/utils.js";
import { isFunction } from "@intlify/shared";
import { getRequestURL, sanitizeStatusCode, setCookie } from "h3";
import { useDetectors } from "../shared/detection.js";
import { domainForHost, domainFromLocale, normalizeDomain } from "../shared/domain.js";
import { isExistingNuxtRoute, matchLocalized } from "../shared/matching.js";
import { createRedirectResolver } from "./utils/redirect.js";
function createRedirectResponse(event, dest, code) {
  event.node.res.setHeader("location", dest);
  event.node.res.statusCode = sanitizeStatusCode(code, event.node.res.statusCode);
  return {
    headers: event.node.res.getHeaders(),
    statusCode: event.node.res.statusCode,
    body: `<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0; url=${dest.replace(/"/g, "%22")}"></head></html>`
  };
}
export default defineNitroPlugin(async (nitro) => {
  const runtimeI18n = useRuntimeI18n();
  const rootRedirect = resolveRootRedirect(runtimeI18n.rootRedirect);
  const _defaultLocale = runtimeI18n.defaultLocale || "";
  try {
    const cacheStorage = useStorage("cache");
    const cachedKeys = await cacheStorage.getKeys("nitro:handlers:i18n");
    await Promise.all(cachedKeys.map((key) => cacheStorage.removeItem(key)));
  } catch {
  }
  const detection = useI18nDetection(void 0);
  const cookieOptions = {
    path: "/",
    domain: detection.cookieDomain || void 0,
    maxAge: 60 * 60 * 24 * 365,
    sameSite: "lax",
    secure: detection.cookieSecure
  };
  const legacyBaseUrl = isFunction(runtimeI18n.baseUrl);
  if (import.meta.dev && legacyBaseUrl) {
    console.warn("[nuxt-i18n] Configuring baseUrl as a function is deprecated and will be removed in v11.");
  }
  const baseUrlGetter = (event) => {
    if (__I18N_DOMAINS__ && !legacyBaseUrl) {
      return domainForHost(runtimeI18n.domainLocales, getRequestURL(event, { xForwardedHost: true })) || "";
    }
    return "";
  };
  const resolveRedirectPath = createRedirectResolver({
    detection,
    rootRedirect,
    redirectStatusCode: runtimeI18n.redirectStatusCode,
    matchLocalized,
    strategy: __I18N_STRATEGY__,
    routing: __I18N_ROUTING__,
    domains: __I18N_DOMAINS__
  });
  const resolveRelocation = (event, locale, path) => {
    const url = getRequestURL(event, { xForwardedHost: true });
    const origin = domainFromLocale(runtimeI18n.domainLocales, url, locale);
    const host = normalizeDomain(origin);
    if (!origin || host === url.host) {
      return;
    }
    const relocated = matchLocalized(path, locale, resolveDefaultLocale(host, _defaultLocale));
    if (!relocated) {
      return;
    }
    return { path: relocated, code: runtimeI18n.redirectStatusCode ?? 302, locale, origin };
  };
  nitro.hooks.hook("request", async (event) => {
    await initializeI18nContext(event);
  });
  nitro.hooks.hook("render:before", async (context) => {
    if (!__I18N_SERVER_REDIRECT__) {
      return;
    }
    const { event } = context;
    const ctx = import.meta.prerender && !event.context.nuxtI18n ? await initializeI18nContext(event) : useI18nContext(event);
    const url = getRequestURL(event);
    const detector = useDetectors(event, detection);
    const localeSegment = detector.route(event.path);
    const pathLocale = isSupportedLocale(localeSegment) && localeSegment || void 0;
    const { pathname } = parsePath(event.path);
    const path = pathLocale ? pathname.slice(pathLocale.length + 1) || "/" : pathname;
    if (!url.pathname.includes(__I18N_SERVER_ROUTE__) && !isExistingNuxtRoute(path)) {
      return;
    }
    const resolved = resolveRedirectPath(
      event.path,
      path,
      pathLocale,
      ctx.vueI18nOptions.defaultLocale,
      detector,
      __I18N_DOMAINS__ ? (locale) => resolveRelocation(event, locale, path) : void 0
    );
    if (resolved.path && (resolved.origin || resolved.path !== pathname)) {
      ctx.detectLocale = resolved.locale;
      detection.useCookie && (!resolved.origin || detection.cookieDomain) && setCookie(event, detection.cookieKey, resolved.locale, cookieOptions);
      context.response = createRedirectResponse(
        event,
        // the resolved path is base-free (matched against base-free routes), re-add `app.baseURL`
        joinURL(
          resolved.origin || baseUrlGetter(event),
          useRuntimeConfig(event).app.baseURL,
          resolved.path + url.search
        ),
        resolved.code
      );
      return;
    }
  });
  nitro.hooks.hook("render:html", (htmlContext, { event }) => {
    const ctx = tryUseI18nContext(event);
    if (__I18N_PRELOAD__) {
      if (ctx == null || Object.keys(ctx.messages ?? {}).length == 0) {
        return;
      }
      if (__I18N_STRIP_UNUSED__ && !import.meta.prerender) {
        const trackedLocales = Object.keys(ctx.trackMap);
        for (const locale of Object.keys(ctx.messages)) {
          if (!trackedLocales.includes(locale)) {
            ctx.messages[locale] = {};
            continue;
          }
          const usedKeys = Array.from(ctx.trackMap[locale]);
          ctx.messages[locale] = pickNested(usedKeys, ctx.messages[locale]);
        }
      }
      const payload = stringifyMessages(ctx.messages);
      if (payload != null) {
        htmlContext.bodyAppend.unshift(
          `<script type="application/json" data-nuxt-i18n="${appId}">${payload}<\/script>`
        );
      }
    }
    if (__I18N_STRICT_SEO__) {
      const raw = JSON.stringify(ctx?.slp ?? {});
      const safe = raw.replace(/</g, "\\u003c").replace(/\u2028/g, "\\u2028").replace(/\u2029/g, "\\u2029");
      htmlContext.head.push(`<script type="application/json" data-nuxt-i18n-slp="${appId}">${safe}<\/script>`);
    }
  });
  if (localeDetector != null) {
    const options = await setupVueI18nOptions(_defaultLocale);
    const i18nMiddleware = defineI18nMiddleware({
      ...options,
      locale: createUserLocaleDetector(options.locale, options.fallbackLocale)
    });
    nitro.hooks.hook("request", i18nMiddleware.onRequest);
    nitro.hooks.hook("afterResponse", i18nMiddleware.onAfterResponse);
  }
});
function stringifyMessages(messages) {
  try {
    return stringify(messages);
  } catch {
    const safe = {};
    for (const locale of Object.keys(messages)) {
      try {
        stringify(messages[locale]);
        safe[locale] = messages[locale];
      } catch (e) {
        console.warn(`[nuxt-i18n] Dropped locale "${locale}" from the preload payload`, e);
      }
    }
    try {
      return stringify(safe);
    } catch (e) {
      console.warn("[nuxt-i18n] Could not serialize the preload payload", e);
    }
  }
}
