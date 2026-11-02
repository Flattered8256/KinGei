import type { Locale } from 'vue-i18n';
import type { RouteLocationPathRaw, RouteLocationResolvedGeneric, RouteRecordNameGeneric, Router } from 'vue-router';
import type { PrefixableOptions } from '#i18n-kit/routing';
/**
 * Returns a getter function which returns a localized route name for the given route and locale,
 * resolved against the routes that were generated rather than from the strategy.
 */
export declare function createLocaleRouteNameGetter(hasRoute: (name: string) => boolean, config: Pick<PrefixableOptions, 'routing' | 'domains'>): (name: RouteRecordNameGeneric | null, locale: string) => string;
/**
 * Factory function which returns a resolver function based on the routing strategy.
 */
export declare function createLocalizedRouteByPathResolver(router: Router, config: PrefixableOptions): (route: RouteLocationPathRaw, locale: Locale) => RouteLocationPathRaw | RouteLocationResolvedGeneric;
