import { assign } from "@intlify/shared";
import { getLocalizedRouteName, normalizeRouteName } from "#i18n-kit/routing";
export function createLocaleRouteNameGetter(hasRoute, config) {
  if (!config.routing && !config.domains) {
    return (routeName) => normalizeRouteName(routeName);
  }
  return (name, locale) => {
    const normalized = normalizeRouteName(name);
    const suffixed = getLocalizedRouteName(normalized, locale, true);
    return hasRoute(suffixed) ? suffixed : getLocalizedRouteName(normalized, locale, false);
  };
}
export function createLocalizedRouteByPathResolver(router, config) {
  if (!config.routing) {
    return (route) => route;
  }
  if (config.strategy === "prefix") {
    return (route, locale) => {
      const targetPath = "/" + locale + (route.path === "/" ? "" : route.path);
      const _route = router.options.routes.find((r) => r.path === targetPath);
      if (_route != null) {
        return router.resolve(assign({}, route, _route, { path: targetPath }));
      }
      return route;
    };
  }
  return (route) => router.resolve(route);
}
