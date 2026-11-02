import { useRouter } from "#imports";
import { defaultRouteNameSuffix, getLocaleFromRouteName } from "#i18n-kit/routing";
export function setupMultiDomainLocales(defaultLocale, strategy, router = useRouter()) {
  if (strategy !== "prefix_except_default" && strategy !== "prefix_and_default") {
    return;
  }
  const keepsDefaultVariant = strategy === "prefix_and_default" && router.getRoutes().some((route) => {
    const routeName = String(route.name);
    return routeName.endsWith(defaultRouteNameSuffix) && getLocaleFromRouteName(routeName) === defaultLocale;
  });
  for (const route of router.getRoutes()) {
    const routeName = String(route.name);
    const locale = getLocaleFromRouteName(routeName);
    if (routeName.endsWith(defaultRouteNameSuffix)) {
      if (!keepsDefaultVariant || locale !== defaultLocale) {
        router.removeRoute(routeName);
      }
      continue;
    }
    if (!keepsDefaultVariant && locale === defaultLocale) {
      router.addRoute({ ...route, path: route.path.replace(new RegExp(`^/${locale}/?`), "/") });
    }
  }
}
