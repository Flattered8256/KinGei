import type { Locale } from 'vue-i18n';
import type { RouteLocationNamedRaw, RouteLocationPathRaw, RouteLocationRaw } from 'vue-router';
import type { CompatRoute } from '../types.js';
import type { RoutingContext } from './context.js';
export type RouteLikeWithPath = RouteLocationPathRaw & {
    name?: string;
};
export type RouteLikeWithName = RouteLocationNamedRaw & {
    path?: string;
};
export type RouteLike = RouteLikeWithPath | RouteLikeWithName;
/**
 * Resolves a localized path of the passed in route.
 */
export declare function localePath(ctx: RoutingContext, route: RouteLocationRaw, locale?: Locale): string;
/**
 * Resolves a localized variant of the passed route.
 */
export declare function localeRoute(ctx: RoutingContext, route: RouteLocationRaw, locale?: Locale): import("vue-router").RouteLocationResolvedGeneric | undefined;
/**
 * Resolve the localized path of the current route.
 */
export declare function switchLocalePath(ctx: RoutingContext, locale: Locale, route?: CompatRoute): string;
