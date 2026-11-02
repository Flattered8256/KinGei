import { hasProtocol, isEqual } from "ufo";
import { getLocalizedRouteName, getRouteBaseName } from "#i18n-kit/routing";
export function createNavigationResolver(config) {
  const { strategy, compactRoutes } = config;
  function isUnlocalizedRoute(to) {
    if (strategy === "no_prefix" || to.name == null) {
      return false;
    }
    const name = String(to.name);
    if (getRouteBaseName(name) !== name) {
      return false;
    }
    if (compactRoutes && to.matched.some((r) => r.meta.__i18nCompact)) {
      return false;
    }
    return !config.getLocaleCodes().some((code) => config.hasRoute(getLocalizedRouteName(name, code, false)));
  }
  return function resolveNavigation(to, locale, pendingLocale = false) {
    const reach = config.localeReach?.(locale);
    if (reach === "off-host") {
      return;
    }
    if (reach === "other-domain") {
      if (isUnlocalizedRoute(to) || pendingLocale) {
        return;
      }
      const destination2 = config.switchLocalePath(locale, to);
      if (!destination2 || !hasProtocol(destination2, { strict: true })) {
        return;
      }
      return { path: destination2, code: config.redirectStatusCode, external: true };
    }
    if (to.path === "/" && config.rootRedirect) {
      return { path: config.localePath(config.rootRedirect.path, locale), code: config.rootRedirect.code };
    }
    if (isUnlocalizedRoute(to)) {
      return;
    }
    if (pendingLocale) {
      return;
    }
    if (config.routeLocale(to) === locale) {
      return;
    }
    const destination = config.switchLocalePath(locale, to) || config.localePath(to.fullPath, locale);
    if (isEqual(destination, to.fullPath)) {
      return;
    }
    return { path: destination, code: config.redirectStatusCode };
  };
}
