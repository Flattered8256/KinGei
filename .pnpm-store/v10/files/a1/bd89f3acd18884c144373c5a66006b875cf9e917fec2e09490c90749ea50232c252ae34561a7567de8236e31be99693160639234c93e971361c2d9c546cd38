import type { RouteLocationPathRaw, RouteRecordNameGeneric, Router } from 'vue-router';
import type { NormalizedLocaleObject, Strategies } from '#internal-i18n-types';
import type { RouteLike } from './routing.js';
import type { I18nRouteMeta, RouteLocationGenericPath } from '../types.js';
/**
 * Platform-neutral context used by the routing functions (`localePath`,
 * `localeRoute`, `switchLocalePath`, `getRouteBaseName`).
 *
 * @internal
 */
export type RoutingContext = {
    router: Router;
    getLocale: () => string;
    getLocales: () => NormalizedLocaleObject[];
    getBaseUrl: (locale?: string) => string;
    /** Extracts the route base name (without locale suffix) */
    getRouteBaseName: (route: RouteRecordNameGeneric | RouteLocationGenericPath | null) => string | undefined;
    /** Modifies the resolved localized path. Middleware for `switchLocalePath` */
    afterSwitchLocalePath: (path: string, locale: string) => string;
    /** `afterSwitchLocalePath` for alternate links, shaped for the locale's canonical domain */
    getAlternatePath: (path: string, locale: string) => string;
    /** Provides localized dynamic parameters for the current route */
    getLocalizedDynamicParams: (locale: string) => Record<string, unknown> | false | undefined;
    /** Prepares a route object to be resolved as a localized route */
    resolveLocalizedRouteObject: (route: RouteLike, locale: string) => RouteLike;
    getRouteLocalizedParams: () => Partial<I18nRouteMeta>;
};
/**
 * Explicit inputs from which a {@link RoutingContext} is constructed, so the
 * context can be created without a Nuxt app (e.g. in unit tests with a
 * memory-history router).
 *
 * @internal
 */
export interface RoutingContextOptions {
    router: Router;
    /** The current host's unprefixed locale */
    defaultLocale: string;
    /** Needed to shape a link for another domain, where the current host's `defaultLocale` says nothing */
    configuredDefaultLocale: string;
    getLocale: () => string;
    getLocales: () => NormalizedLocaleObject[];
    getBaseUrl: (locale?: string) => string;
    getCanonicalBaseUrl: (locale: string) => string;
    /** Host of the current request/page, used for domain-based behavior */
    getHost: () => string | undefined;
    /**
     * Returns the `switchLocalePath` payload when it takes precedence over route
     * meta (strict SEO client-side hydration), a falsy value otherwise.
     */
    getLocalePathPayload?: () => Record<string, Record<string, string> | false> | false | undefined;
    strategy: Strategies;
    /** Whether routes are localized (pages enabled and strategy is not `no_prefix`) */
    routing: boolean;
    /** Whether locales are resolved from domains */
    domains: boolean;
    trailingSlash: boolean;
    strictSeo: boolean;
    compactRoutes: boolean;
}
export declare const isRouteLocationPathRaw: (val: RouteLike) => val is RouteLocationPathRaw;
export declare function createRoutingContext(options: RoutingContextOptions): RoutingContext;
