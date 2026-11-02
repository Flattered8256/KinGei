import { parsePath } from "ufo";
import { createLocaleDetector } from "../../shared/detection.js";
import { isSupportedLocale } from "../../shared/locales.js";
export function createRedirectResolver(config) {
  const { detection, rootRedirect, matchLocalized, strategy, routing, domains } = config;
  const isSupported = config.isSupportedLocale ?? isSupportedLocale;
  const detectLocale = createLocaleDetector({ detection, isSupportedLocale: isSupported, routing, domains });
  return function resolveRedirectPath(fullPath, path, pathLocale, defaultLocale, detectors, relocate) {
    if (routing && domains && pathLocale && !detectors.onHost(pathLocale)) {
      const relocated = relocate?.(pathLocale);
      if (relocated) {
        return relocated;
      }
    }
    let locale = detectLocale(detectors, fullPath, true) || defaultLocale;
    function getLocalizedMatch(locale2) {
      const res = matchLocalized(path || "/", locale2, defaultLocale);
      if (res && res !== fullPath) {
        return res;
      }
    }
    let resolvedPath = void 0;
    let redirectCode = 302;
    const pathname = parsePath(fullPath).pathname;
    if (rootRedirect && pathname === "/") {
      locale = detection.enabled && locale || defaultLocale;
      resolvedPath = isSupported(detectors.route(rootRedirect.path)) && rootRedirect.path || matchLocalized(rootRedirect.path, locale, defaultLocale);
      redirectCode = rootRedirect.code;
    } else if (config.redirectStatusCode) {
      redirectCode = config.redirectStatusCode;
    }
    switch (detection.redirectOn) {
      case "root":
        if (pathname !== "/") {
          break;
        }
      // fallthrough (root has no prefix)
      case "no prefix":
        if (pathLocale) {
          break;
        }
      // fallthrough to resolve
      case "all":
        resolvedPath ??= getLocalizedMatch(locale);
        break;
    }
    if (pathname === "/" && strategy === "prefix") {
      resolvedPath ??= getLocalizedMatch(defaultLocale);
    }
    if (domains && !detectors.onHost(locale)) {
      const relocated = relocate?.(locale);
      if (relocated) {
        return relocated;
      }
    }
    return { path: resolvedPath, code: redirectCode, locale };
  };
}
