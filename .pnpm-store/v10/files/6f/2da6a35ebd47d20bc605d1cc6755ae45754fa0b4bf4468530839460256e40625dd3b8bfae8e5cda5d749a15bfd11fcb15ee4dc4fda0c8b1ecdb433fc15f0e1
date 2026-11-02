import { joinURL } from "ufo";
import { createLocaleRouteNameGetter, createLocalizedRouteByPathResolver } from "./utils.js";
import { createTrailingSlashFormatter, getLocaleFromRoutePath, getRouteBaseName, prefixable } from "#i18n-kit/routing";
import { isSupportedLocale, resolveDefaultLocale } from "../shared/locales.js";
import { canonicalDomain, isLocaleOnHost, normalizeDomain } from "../shared/domain.js";
export const isRouteLocationPathRaw = (val) => !!val.path && !val.name;
export function createRoutingContext(options) {
  const { router, defaultLocale, strictSeo, compactRoutes } = options;
  const config = {
    strategy: options.strategy,
    routing: options.routing,
    domains: options.domains
  };
  const formatTrailingSlash = createTrailingSlashFormatter(options.trailingSlash);
  const getLocalizedRouteName = createLocaleRouteNameGetter((name) => router.hasRoute(name), config);
  function resolveLocalizedRouteByName(route, locale) {
    route.name = getRouteBaseName(route.name || router.currentRoute.value);
    const localizedName = getLocalizedRouteName(route.name, locale);
    if (router.hasRoute(localizedName)) {
      route.name = localizedName;
      if (compactRoutes && route.params) {
        delete route.params.locale;
      }
    } else if (compactRoutes && isSupportedLocale(locale) && getCompactRouteNames().has(route.name)) {
      route.params = { ...route.params || {}, locale };
      return route;
    }
    route.name = localizedName;
    return route;
  }
  const routeByPathResolver = createLocalizedRouteByPathResolver(router, config);
  let compactRouteRecords;
  function getCompactRouteNames() {
    if (compactRouteRecords) {
      return compactRouteRecords;
    }
    compactRouteRecords = /* @__PURE__ */ new Set();
    if (compactRoutes) {
      for (const r of router.getRoutes()) {
        if (r.name != null && /^\/:locale\(/.test(r.path)) {
          compactRouteRecords.add(String(r.name));
        }
      }
    }
    return compactRouteRecords;
  }
  function resolveLocalizedRouteByPath(input, locale) {
    const route = routeByPathResolver(input, locale);
    const baseName = getRouteBaseName(route);
    if (baseName) {
      const localizedName = getLocalizedRouteName(baseName, locale);
      if (router.hasRoute(localizedName)) {
        route.name = localizedName;
        const named = route;
        if (compactRoutes && named.params) {
          delete named.params.locale;
        }
        return route;
      }
      if (compactRoutes && getCompactRouteNames().has(baseName)) {
        const compacted = route;
        compacted.name = baseName;
        compacted.params = { ...compacted.params || {}, locale };
        return compacted;
      }
      route.name = localizedName;
      return route;
    }
    if (prefixable(locale, defaultLocale, config)) {
      route.path = "/" + locale + route.path;
    }
    route.path = formatTrailingSlash(route.path, true);
    return route;
  }
  const getRouteLocalizedParams = () => router.currentRoute.value.meta[__DYNAMIC_PARAMS_KEY__] ?? {};
  const missesLocalizedParams = (locale, params = getRouteLocalizedParams()) => strictSeo && !!locale && Object.keys(params).length > 0 && !params[locale];
  const stripsDefaultPrefix = config.strategy === "prefix_except_default" || config.strategy === "prefix_and_default";
  const unprefixesLocale = (domain, locale, locales) => !!domain && resolveDefaultLocale(normalizeDomain(domain), options.configuredDefaultLocale, locales) === locale;
  const toDomainUrl = (getBase) => (path, locale, target, locales) => {
    const base = getBase(locale);
    if (!base) {
      return path;
    }
    if (stripsDefaultPrefix && unprefixesLocale(canonicalDomain(target), locale, locales) && getLocaleFromRoutePath(path) === locale) {
      path = path.slice(locale.length + 1) || "/";
    }
    return joinURL(base, path);
  };
  const toServingUrl = toDomainUrl(options.getBaseUrl);
  const toCanonicalUrl = toDomainUrl(options.getCanonicalBaseUrl);
  return {
    router,
    getLocale: options.getLocale,
    getLocales: options.getLocales,
    getBaseUrl: options.getBaseUrl,
    getRouteBaseName,
    getRouteLocalizedParams,
    getLocalizedDynamicParams: (locale) => {
      const payload = options.getLocalePathPayload?.();
      if (payload) {
        return payload[locale] || {};
      }
      return getRouteLocalizedParams()?.[locale];
    },
    afterSwitchLocalePath: (path, locale) => {
      if (missesLocalizedParams(locale)) {
        return "";
      }
      if (!config.domains) {
        return path;
      }
      const host = options.getHost() ?? "";
      const locales = options.getLocales();
      const target = locales.find((l) => l.code === locale);
      if (isLocaleOnHost(target, host)) {
        return path;
      }
      if (!path) {
        return path;
      }
      return toServingUrl(path, locale, target, locales);
    },
    getAlternatePath: (path, locale) => {
      if (locale !== options.getLocale() && missesLocalizedParams(locale)) {
        return "";
      }
      if (!config.domains || !path) {
        return path;
      }
      const locales = options.getLocales();
      return toCanonicalUrl(path, locale, locales.find((l) => l.code === locale), locales);
    },
    resolveLocalizedRouteObject: (route, locale) => {
      return isRouteLocationPathRaw(route) ? resolveLocalizedRouteByPath(route, locale) : resolveLocalizedRouteByName(route, locale);
    }
  };
}
