import type { Strategies } from '#internal-i18n-types';
import type { CompatRoute } from '../types.js';
export type NavigationResolverConfig = {
    rootRedirect?: {
        path: string;
        code: number;
    };
    redirectStatusCode?: number;
    /** Resolves a localized path, e.g. `localePath` */
    localePath: (to: string, locale: string) => string;
    /** Resolves the localized variant of the route, e.g. `switchLocalePath` */
    switchLocalePath: (locale: string, to: CompatRoute) => string;
    /** Locale detected from the route path/name */
    routeLocale: (to: CompatRoute) => string | undefined;
    hasRoute: (name: string) => boolean;
    getLocaleCodes: () => string[];
    /** How the current host can reach a locale (see `resolveLocaleReach`), set under domain setups */
    localeReach?: (locale: string) => 'here' | 'other-domain' | 'off-host';
    strategy: Strategies;
    compactRoutes: boolean;
};
export type ResolvedNavigation = {
    path: string;
    code: number | undefined;
    external?: boolean;
};
export declare function createNavigationResolver(config: NavigationResolverConfig): (to: CompatRoute, locale: string, pendingLocale?: boolean) => ResolvedNavigation | undefined;
