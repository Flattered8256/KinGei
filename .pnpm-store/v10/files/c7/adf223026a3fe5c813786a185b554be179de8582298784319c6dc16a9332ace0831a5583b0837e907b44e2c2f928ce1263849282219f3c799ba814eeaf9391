import { withTrailingSlash } from 'ufo';
import type { Strategies } from '#internal-i18n-types';
import type { RouteName, RouteObject } from './types.js';
export declare const defaultRouteNameSuffix: any;
/**
 * Normalizes {@link RouteName} to string
 */
export declare function normalizeRouteName(routeName: RouteName): string;
/**
 * Extract route name without localization from {@link RouteName} or {@link RouteObject}
 */
export declare function getRouteBaseName(route: RouteName | RouteObject): string | undefined;
export declare function getLocalizedRouteName(routeName: string, locale: string, isDefault: boolean): string;
export declare const createTrailingSlashFormatter: (trailingSlash: boolean) => typeof withTrailingSlash;
export type PrefixableOptions = {
    strategy: Strategies;
    /** Whether routes are localized (pages enabled and strategy is not `no_prefix`) */
    routing: boolean;
    /** Whether locales are resolved from domains */
    domains: boolean;
};
export declare function prefixable(currentLocale: string, defaultLocale: string, options: Pick<PrefixableOptions, 'strategy' | 'routing'>): boolean;
export declare const getLocaleFromRoutePath: (path: string) => string;
export declare const getLocaleFromRouteName: (name: string) => string;
/**
 * Extract locale code from route name or path
 */
export declare function getLocaleFromRoute(route: RouteName | RouteObject): string;
