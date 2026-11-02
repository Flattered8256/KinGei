import { getCookie, getRequestHeader, getRequestURL } from "h3";
import { parsePath } from "ufo";
import { normalizedLocales } from "#build/i18n-options.mjs";
import { getLocaleFromRoute, getLocaleFromRoutePath } from "#i18n-kit/routing";
import { findBrowserLocale } from "#i18n-kit/browser";
import { parseAcceptLanguage } from "@intlify/utils";
import { cookieSpansDomains, isLocaleOnHost, isLocaleServedOnHost, matchDomainLocale, withRuntimeDomain } from "./domain.js";
import { isString } from "@intlify/shared";
import { isSupportedLocale } from "./locales.js";
import { useRuntimeI18n } from "../shared/utils.js";
import { useCookie } from "#app";
const getCookieLocale = (event, cookieName) => (import.meta.client ? useCookie(cookieName).value : getCookie(event, cookieName)) || void 0;
const getRouteLocale = (event, route) => getLocaleFromRoute(route);
const getHeaderLocale = (event) => findBrowserLocale(normalizedLocales, parseAcceptLanguage(getRequestHeader(event, "accept-language") || ""));
const getNavigatorLocale = (event) => findBrowserLocale(normalizedLocales, navigator.languages);
const getRequestHost = (event) => import.meta.client ? new URL(window.location.href).host : getRequestURL(event, { xForwardedHost: true }).host;
const getRefererHost = (event) => {
  const referer = import.meta.client ? document.referrer : getRequestHeader(event, "referer");
  try {
    return referer && new URL(referer).host || void 0;
  } catch {
    return void 0;
  }
};
const getDomainLocales = (domainLocales) => normalizedLocales.map((l) => withRuntimeDomain(l, domainLocales));
export const useDetectors = (event, config, nuxtApp) => {
  if (import.meta.server && !event) {
    throw new Error("H3Event is required for server-side locale detection");
  }
  const runtimeI18n = useRuntimeI18n(nuxtApp);
  let host;
  let locales;
  const getHost = () => host ??= getRequestHost(event);
  const getLocales = () => locales ??= getDomainLocales(runtimeI18n.domainLocales);
  return {
    cookie: () => getCookieLocale(event, config.cookieKey),
    header: () => import.meta.server ? getHeaderLocale(event) : void 0,
    navigator: () => import.meta.client ? getNavigatorLocale(event) : void 0,
    host: (path) => matchDomainLocale(getLocales(), getHost(), getLocaleFromRoutePath(path)),
    route: (path) => getRouteLocale(event, path),
    /** Passes the locale through when the current host serves it, `undefined` otherwise */
    onHost: (locale) => !locale || isLocaleServedOnHost(getLocales(), getHost(), locale) ? locale : void 0,
    /** Whether the visitor arrived from one of the configured domains */
    fromOwnDomain: () => {
      const referer = getRefererHost(event);
      return !!referer && getLocales().some((l) => isLocaleOnHost(l, referer));
    },
    /** Whether a cookie scoped to the configured `cookieDomain` is readable on every domain */
    cookieSpans: () => !!config.cookieDomain && cookieSpansDomains(getLocales(), config.cookieDomain)
  };
};
export function createLocaleDetector(config) {
  const { detection, routing, domains } = config;
  const isSupported = config.isSupportedLocale ?? isSupportedLocale;
  function skipDetect(path, pathLocale) {
    if (!routing) {
      return false;
    }
    if (detection.redirectOn === "root" && path !== "/") {
      return true;
    }
    if (detection.redirectOn === "no prefix" && !detection.alwaysRedirect && isSupported(pathLocale)) {
      return true;
    }
    return false;
  }
  return function detectLocale(detectors, route, initial) {
    const path = isString(route) ? parsePath(route).pathname : route.path;
    const pass = (locale) => locale;
    const onHost = domains ? detectors.onHost : pass;
    function* detect() {
      const detecting = initial && detection.enabled && !skipDetect(path, detectors.route(path));
      if (detecting) {
        const external = domains && !detectors.fromOwnDomain();
        const cookie = external && detectors.cookieSpans() ? pass : onHost;
        const browser = external ? pass : onHost;
        yield cookie(detectors.cookie());
        yield browser(detectors.header());
        yield browser(detectors.navigator());
      }
      if (domains) {
        yield detectors.host(path);
      }
      if (routing) {
        yield detectors.route(route);
      }
      if (detecting) {
        yield onHost(detection.fallbackLocale);
      }
    }
    for (const detected of detect()) {
      if (detected && isSupported(detected)) {
        return detected;
      }
    }
    return "";
  };
}
